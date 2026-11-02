//! Day/night brightness schedule driven by sunrise/sunset times.
//!
//! Pure solar math (Wikipedia "sunrise equation", a simplified NOAA model)
//! computes sunrise/sunset for the configured location — no network access
//! needed. A tokio engine periodically evaluates the current phase (night,
//! day, evening) and applies the configured brightness to all monitors.
//!
//! Phase timeline for one day (times are local):
//!
//! ```text
//!   midnight ────── sunrise ────── sunset ────── sunset + evening_duration ── midnight
//!      Night          Day           Evening              Night
//!      (15%)          (90%)          (45%)                (15%)
//! ```
//!
//! Each transition ramps linearly over `ramp_minutes` instead of jumping.

use crate::core::config::{ConfigManager, ScheduleConfig};
use crate::ddc::DDCManager;
use chrono::{DateTime, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// Julian date of the J2000.0 epoch (2000-01-01 12:00 UTC).
const J2000: f64 = 2_451_545.0;
/// Earth's axial tilt in degrees.
const AXIAL_TILT_DEG: f64 = 23.44;
/// Sun altitude at sunrise/sunset in degrees: the geometric center of the
/// sun is at −0.833° (accounts for atmospheric refraction + solar disk).
const SUNRISE_ALTITUDE_DEG: f64 = -0.833;
/// Longitude of perihelion (degrees), part of the ecliptic longitude.
const PERIHELION_LONGITUDE_DEG: f64 = 102.937_2;

/// Phase of the day used for brightness selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// After sunset + evening duration, until sunrise.
    Night,
    /// Between sunrise and sunset.
    Day,
    /// Between sunset and sunset + evening duration.
    Evening,
}

impl Phase {
    /// Brightness configured for this phase.
    pub fn brightness(self, cfg: &ScheduleConfig) -> u16 {
        match self {
            Phase::Night => cfg.night_brightness,
            Phase::Day => cfg.day_brightness,
            Phase::Evening => cfg.evening_brightness,
        }
    }

    /// Human-readable phase name.
    pub fn name(self) -> &'static str {
        match self {
            Phase::Night => "Night",
            Phase::Day => "Day",
            Phase::Evening => "Evening",
        }
    }
}

/// A brightness change between two phases at a point in time.
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    /// When the transition begins (local time).
    pub at: DateTime<Local>,
    /// Phase before the transition.
    pub from: Phase,
    /// Phase after the transition.
    pub to: Phase,
}

/// Result of evaluating the schedule at a point in time.
#[derive(Debug, Clone)]
pub struct ScheduleDecision {
    /// Brightness target right now (before min_brightness clamping).
    pub target: u16,
    /// Current phase.
    pub phase: Phase,
    /// True while ramping between phases.
    pub ramping: bool,
    /// Next transition (when it fires and which phase it brings).
    pub next: Option<(DateTime<Local>, Phase)>,
}

impl ScheduleDecision {
    /// One-line status for the tray menu, e.g.
    /// `Day 90% — until 16:24 (Wrocław, Poland)`.
    pub fn status_line(&self, cfg: &ScheduleConfig) -> String {
        let next_info = match &self.next {
            Some((at, to)) => format!("until {} {}", at.format("%H:%M"), to.name()),
            None => "no upcoming transition".to_string(),
        };
        let prefix = if self.ramping {
            "Ramping → ".to_string()
        } else {
            String::new()
        };
        format!(
            "{}{} {}% — {} ({})",
            prefix,
            self.phase.name(),
            self.target,
            next_info,
            cfg.location_name
        )
    }
}

// ─── Solar math ─────────────────────────────────────────────────────────────

fn sin_deg(deg: f64) -> f64 {
    deg.to_radians().sin()
}

fn cos_deg(deg: f64) -> f64 {
    deg.to_radians().cos()
}

fn acos_deg(x: f64) -> f64 {
    x.clamp(-1.0, 1.0).acos().to_degrees()
}

/// Julian date from a Unix timestamp in seconds.
fn julian_from_unix_secs(secs: i64) -> f64 {
    secs as f64 / 86_400.0 + 2_440_587.5
}

/// Unix timestamp seconds from a Julian date.
fn unix_secs_from_julian(jd: f64) -> i64 {
    ((jd - 2_440_587.5) * 86_400.0).round() as i64
}

/// Compute sunrise/sunset (UTC naive datetimes) for a local civil date.
///
/// `local_offset_secs` is the UTC offset of that date's local time in seconds
/// (e.g. +3600 for CET, +7200 for CEST). Returns `None` for polar
/// day/night, when the sun never crosses the sunrise zenith.
///
/// Reference values (sunrise-sunset.org API, queried 2026-11-02, Wrocław
/// 51.1079°N 17.0385°E):
/// - 2026-11-02 (CET, +01:00): sunrise 05:46:05 UTC, sunset 15:24:43 UTC
/// - 2026-06-21 (CEST, +02:00): sunrise 02:36:36 UTC, sunset 19:10:42 UTC
pub fn sun_times_utc(
    date: NaiveDate,
    local_offset_secs: i32,
    latitude: f64,
    longitude: f64,
) -> Option<(NaiveDateTime, NaiveDateTime)> {
    // The instant of local noon expressed as a UTC naive datetime. Used only
    // to pick the right Julian cycle; sunrise/sunset instants themselves are
    // computed on the UTC timeline, so a ±1h offset error near a DST switch
    // cannot shift the result by a day.
    let local_noon = date.and_hms_opt(12, 0, 0)?;
    let noon_utc = local_noon - Duration::seconds(local_offset_secs as i64);
    let jd_noon = julian_from_unix_secs(noon_utc.and_utc().timestamp());

    // Current Julian cycle (day) since J2000.
    let n = (jd_noon - J2000 + 0.000_8).ceil();

    // Longitude west is positive in this formulation.
    let lw = -longitude;

    // Approximate solar noon (Julian date).
    let j_star = J2000 + 0.000_9 + lw / 360.0 + n;

    // Solar mean anomaly (degrees).
    let m = (357.529_1 + 0.985_600_28 * (j_star - J2000)).rem_euclid(360.0);

    // Equation of the center (degrees).
    let c = 1.914_8 * sin_deg(m) + 0.020_0 * sin_deg(2.0 * m) + 0.000_3 * sin_deg(3.0 * m);

    // Ecliptic longitude (degrees): true anomaly + 180° + perihelion longitude.
    let lambda = (m + c + 180.0 + PERIHELION_LONGITUDE_DEG).rem_euclid(360.0);

    // Solar transit (Julian date).
    let j_transit = j_star + 0.005_3 * sin_deg(m) - 0.006_9 * sin_deg(2.0 * lambda);

    // Sun declination (degrees).
    let sin_delta = sin_deg(lambda) * sin_deg(AXIAL_TILT_DEG);
    let delta = sin_delta.asin().to_degrees();

    // Hour angle for the sunrise altitude (degrees).
    let cos_omega = (sin_deg(SUNRISE_ALTITUDE_DEG) - sin_deg(latitude) * sin_deg(delta))
        / (cos_deg(latitude) * cos_deg(delta));
    if !(-1.0..=1.0).contains(&cos_omega) {
        // Polar day or polar night: no sunrise/sunset on this date.
        return None;
    }
    let omega = acos_deg(cos_omega);

    let jd_sunrise = j_transit - omega / 360.0;
    let jd_sunset = j_transit + omega / 360.0;

    let sunrise = DateTime::from_timestamp(unix_secs_from_julian(jd_sunrise), 0)?.naive_utc();
    let sunset = DateTime::from_timestamp(unix_secs_from_julian(jd_sunset), 0)?.naive_utc();
    Some((sunrise, sunset))
}

/// Sunrise/sunset for a local date, expressed as local-time instants.
pub fn sun_times_local(
    date: NaiveDate,
    local_offset_secs: i32,
    latitude: f64,
    longitude: f64,
) -> Option<(DateTime<Local>, DateTime<Local>)> {
    let (sunrise_utc, sunset_utc) = sun_times_utc(date, local_offset_secs, latitude, longitude)?;
    let sunrise = Utc.from_utc_datetime(&sunrise_utc).with_timezone(&Local);
    let sunset = Utc.from_utc_datetime(&sunset_utc).with_timezone(&Local);
    Some((sunrise, sunset))
}

// ─── Phase logic ────────────────────────────────────────────────────────────

/// All transitions for one local date (sunrise, sunset, evening→night).
fn transitions_for_date(
    date: NaiveDate,
    offset: FixedOffset,
    cfg: &ScheduleConfig,
) -> Vec<Transition> {
    let mut out = Vec::new();
    if let Some((sunrise, sunset)) =
        sun_times_local(date, offset.local_minus_utc(), cfg.latitude, cfg.longitude)
    {
        out.push(Transition {
            at: sunrise,
            from: Phase::Night,
            to: Phase::Day,
        });
        out.push(Transition {
            at: sunset,
            from: Phase::Day,
            to: Phase::Evening,
        });
        out.push(Transition {
            at: sunset + Duration::minutes(cfg.evening_duration_min as i64),
            from: Phase::Evening,
            to: Phase::Night,
        });
    }
    out
}

/// Compute the schedule decision at `now`.
///
/// The initial phase (before any transition) is Night — the darkest and most
/// conservative default for pre-sunrise times with missing solar data.
pub fn decide(now: DateTime<Local>, cfg: &ScheduleConfig) -> ScheduleDecision {
    let offset = now.offset().to_owned();
    let today = now.date_naive();

    // Collect transitions from yesterday through tomorrow so any `now`
    // finds its surrounding events (covers late-evening/early-morning edges).
    let mut transitions = Vec::new();
    for days in -1..=1 {
        let Some(d) = today.checked_add_signed(Duration::days(days)) else {
            continue;
        };
        transitions.extend(transitions_for_date(d, offset.clone(), cfg));
    }
    transitions.sort_by_key(|t| t.at);

    let active = transitions.iter().rev().find(|t| t.at <= now).cloned();
    let next = transitions.iter().find(|t| t.at > now);

    let (phase, target, ramping) = match active {
        None => (Phase::Night, cfg.night_brightness, false),
        Some(t) => {
            let ramp = Duration::minutes(cfg.ramp_minutes as i64);
            if cfg.ramp_minutes > 0
                && now < t.at + ramp
                && t.from.brightness(cfg) != t.to.brightness(cfg)
            {
                // Interpolate linearly from the previous phase's brightness.
                let t_frac = (now - t.at).num_seconds() as f64 / ramp.num_seconds() as f64;
                let from = f64::from(t.from.brightness(cfg));
                let to = f64::from(t.to.brightness(cfg));
                let value = from + (to - from) * t_frac.clamp(0.0, 1.0);
                (t.to, value.round() as u16, true)
            } else {
                (t.to, t.to.brightness(cfg), false)
            }
        }
    };

    ScheduleDecision {
        target,
        phase,
        ramping,
        next: next.map(|t| (t.at, t.to)),
    }
}

// ─── Engine ─────────────────────────────────────────────────────────────────

/// Messages the UI can send to the schedule engine.
#[derive(Debug, Clone, Copy)]
pub enum ScheduleEvent {
    /// The user manually changed brightness — suspend the schedule for the
    /// current phase so their choice sticks until the next transition.
    ManualChange,
}

/// Spawn the schedule engine on the current tokio runtime.
///
/// Ticks every 30 seconds, evaluates the schedule and applies the target
/// brightness to all monitors (clamped by `behavior.min_brightness`). Manual
/// changes suspend the engine until the next phase transition; the tray menu
/// toggle can disable the schedule entirely.
///
/// `tray` receives live status-line updates for the menu.
pub fn spawn_engine(
    ddc: Arc<DDCManager>,
    config_manager: Arc<TokioMutex<ConfigManager>>,
    tray: Option<Arc<ksni::Handle<crate::ui::tray_icon::TrayState>>>,
    event_rx: tokio::sync::mpsc::UnboundedReceiver<ScheduleEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::runtime::Handle::current().spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(30));
        let mut last_applied: Option<u16> = None;
        // Some(phase): user overrode brightness during this phase.
        let mut suspended_for: Option<Phase> = None;
        // Last status string pushed into the tray menu. The tray is only
        // updated when this changes — some shell AppIndicator implementations
        // misbehave when the menu layout is re-sent every tick.
        let mut last_status = String::new();
        let mut event_rx = event_rx;

        loop {
            tokio::select! {
                _ = ticker.tick() => {}
                maybe_event = event_rx.recv() => {
                    match maybe_event {
                        Some(ScheduleEvent::ManualChange) => {
                            let cfg = config_manager.lock().await.config().schedule.clone();
                            if cfg.enabled {
                                let decision = decide(Local::now(), &cfg);
                                suspended_for = Some(decision.phase);
                                tracing::info!(
                                    "Schedule suspended for {} phase (manual brightness change)",
                                    decision.phase.name()
                                );
                            }
                            continue;
                        }
                        // All senders dropped (shutdown) — keep ticking on
                        // the timer until the process exits.
                        None => {
                            let _ = ticker.tick().await;
                            continue;
                        }
                    }
                }
            }

            let (cfg, min_brightness) = {
                let mgr = config_manager.lock().await;
                (
                    mgr.config().schedule.clone(),
                    mgr.config().behavior.min_brightness,
                )
            };

            if !cfg.enabled {
                last_applied = None;
                suspended_for = None;
                if last_status != "off" {
                    if let Some(tray) = tray.as_ref() {
                        let _ = tray
                            .update(|t| {
                                t.schedule_enabled = false;
                                t.schedule_status = "Auto schedule off".to_string();
                            })
                            .await;
                    }
                    last_status = "off".to_string();
                }
                continue;
            }

            let now = Local::now();
            let decision = decide(now, &cfg);
            let status = decision.status_line(&cfg);

            // Push to the tray only when the rendered status actually changed.
            if status != last_status {
                if let Some(tray) = tray.as_ref() {
                    let s = status.clone();
                    let _ = tray
                        .update(|t| {
                            t.schedule_enabled = true;
                            t.schedule_status = s;
                        })
                        .await;
                }
                last_status = status;
            }

            // A manual override is only valid for the phase it was made in.
            if suspended_for == Some(decision.phase) {
                tracing::debug!(
                    "Schedule: respecting manual override for {} phase",
                    decision.phase.name()
                );
                continue;
            }
            if suspended_for.is_some() {
                tracing::info!(
                    "Schedule: phase changed to {}, resuming automatic control",
                    decision.phase.name()
                );
                suspended_for = None;
            }

            let target = decision.target.max(min_brightness);
            if last_applied != Some(target) {
                tracing::info!(
                    "Schedule: applying {} {}% (ramping={})",
                    decision.phase.name(),
                    target,
                    decision.ramping
                );
                if let Err(e) = ddc.set_all_brightness(target, min_brightness).await {
                    tracing::warn!("Schedule: failed to apply brightness: {}", e);
                } else {
                    last_applied = Some(target);
                }
            }
        }
    })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    /// Reference values from the sunrise-sunset.org API for Wrocław
    /// (51.1079°N, 17.0385°E), queried 2026-11-02 with tzid=Europe/Warsaw.
    /// The simplified sunrise equation used here is accurate to a few minutes.
    fn assert_close(got: NaiveDateTime, want: &str, label: &str) {
        let want = NaiveTime::parse_from_str(want, "%H:%M:%S").unwrap();
        let want_dt = NaiveDateTime::new(got.date(), want);
        let diff_min = (got - want_dt).num_minutes().abs();
        println!(
            "{}: got {} vs reference {} (diff {} min)",
            label,
            got.format("%H:%M:%S"),
            want.format("%H:%M:%S"),
            diff_min
        );
        assert!(
            diff_min <= 5,
            "{} differs by {} min from reference (got {}, want {})",
            label,
            diff_min,
            got.format("%H:%M:%S"),
            want.format("%H:%M:%S")
        );
    }

    #[test]
    fn test_sun_times_wroclaw_november() {
        // 2026-11-02, CET (+01:00): sunrise 05:46:05 UTC, sunset 15:24:43 UTC
        let (sunrise, sunset) = sun_times_utc(
            NaiveDate::from_ymd_opt(2026, 11, 2).unwrap(),
            3600,
            51.1079,
            17.0385,
        )
        .expect("sun times for Wrocław in November");
        assert_close(sunrise, "05:46:05", "sunrise 2026-11-02");
        assert_close(sunset, "15:24:43", "sunset 2026-11-02");
    }

    #[test]
    fn test_sun_times_wroclaw_june() {
        // 2026-06-21, CEST (+02:00): sunrise 02:36:36 UTC, sunset 19:10:42 UTC
        let (sunrise, sunset) = sun_times_utc(
            NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
            7200,
            51.1079,
            17.0385,
        )
        .expect("sun times for Wrocław in June");
        assert_close(sunrise, "02:36:36", "sunrise 2026-06-21");
        assert_close(sunset, "19:10:42", "sunset 2026-06-21");
    }

    #[test]
    fn test_sun_times_polar_night() {
        // Svalbard (78.2°N) in December: sun never rises → None.
        let result = sun_times_utc(
            NaiveDate::from_ymd_opt(2026, 12, 21).unwrap(),
            3600,
            78.2,
            15.6,
        );
        assert!(
            result.is_none(),
            "polar night should yield None, got {:?}",
            result
        );
    }

    fn winter_cfg() -> ScheduleConfig {
        ScheduleConfig {
            ramp_minutes: 0,
            evening_duration_min: 60,
            ..ScheduleConfig::default()
        }
    }

    fn winter_tz() -> FixedOffset {
        FixedOffset::east_opt(3600).unwrap()
    }

    fn local_at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Local> {
        // Deterministic regardless of the test machine's TZ: pin the CET
        // offset (+01:00) onto the DateTime<Local> explicitly.
        let naive_local = NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, 0)
            .unwrap();
        let offset = winter_tz();
        let naive_utc = naive_local - Duration::seconds(offset.local_minus_utc() as i64);
        DateTime::<Local>::from_naive_utc_and_offset(naive_utc, offset)
    }

    #[test]
    fn test_decide_phases_no_ramp() {
        let cfg = winter_cfg();

        // 08:00 local on 2026-11-02 (sunrise ~06:46 local): Day
        let d = decide(local_at(2026, 11, 2, 8, 0), &cfg);
        assert_eq!(d.phase, Phase::Day);
        assert_eq!(d.target, 90);
        // Next transition: sunset → Evening. The simplified sunrise equation
        // lands within ~2 min of the reference (sunset 16:24:43 local).
        let (at, to) = d.next.expect("sunset transition must exist");
        assert_eq!(to, Phase::Evening);
        let got_min = at.format("%H:%M").to_string();
        let got_min: i64 =
            got_min[..2].parse::<i64>().unwrap() * 60 + got_min[3..].parse::<i64>().unwrap();
        let ref_min = 16 * 60 + 24; // 16:24 from sunrise-sunset.org
        let diff = (got_min - ref_min).abs();
        println!(
            "sunset transition at {}, ref 16:24 (diff {} min)",
            got_min, diff
        );
        assert!(diff <= 5, "sunset {} min off reference 16:24", diff);

        // 12:00 local: still Day (right after solar noon)
        let d = decide(local_at(2026, 11, 2, 12, 0), &cfg);
        assert_eq!(d.phase, Phase::Day);

        // 17:00 local (after sunset 16:24, before 17:24): Evening
        let d = decide(local_at(2026, 11, 2, 17, 0), &cfg);
        assert_eq!(d.phase, Phase::Evening);
        assert_eq!(d.target, 45);

        // 20:00 local (after sunset+60min): Night
        let d = decide(local_at(2026, 11, 2, 20, 0), &cfg);
        assert_eq!(d.phase, Phase::Night);
        assert_eq!(d.target, 15);

        // 03:00 local (before sunrise): Night
        let d = decide(local_at(2026, 11, 2, 3, 0), &cfg);
        assert_eq!(d.phase, Phase::Night);
    }

    #[test]
    fn test_decide_ramping_interpolates() {
        let mut cfg = winter_cfg();
        cfg.ramp_minutes = 60;
        // Longer evening so the Day→Evening ramp (60 min) completes BEFORE
        // the night transition (sunset + evening_duration_min).
        cfg.evening_duration_min = 120;

        // Mid-ramp, ~30 minutes after sunset (~16:26 local):
        // brightness should be between Day (90) and Evening (45) ≈ 67-70.
        let at = local_at(2026, 11, 2, 16, 54);
        let d = decide(at, &cfg);
        assert_eq!(d.phase, Phase::Evening);
        assert!(d.ramping);
        assert!(
            (65..=70).contains(&d.target),
            "mid-ramp target {} not in [65, 70]",
            d.target
        );

        // After the ramp completes (sunset + 60 min ≈ 17:26) and before
        // night starts (sunset + 120 min ≈ 18:26): exactly Evening.
        let at = local_at(2026, 11, 2, 17, 40);
        let d = decide(at, &cfg);
        assert_eq!(d.phase, Phase::Evening);
        assert!(!d.ramping);
        assert_eq!(d.target, 45);

        // 20:00 local: night started ~18:26 and its 60-min ramp ended
        // ~19:26 — Night phase, stable value.
        let at = local_at(2026, 11, 2, 20, 0);
        let d = decide(at, &cfg);
        assert_eq!(d.phase, Phase::Night);
        assert!(!d.ramping);
        assert_eq!(d.target, 15);
    }

    #[test]
    fn test_status_line_format() {
        let cfg = winter_cfg();
        let d = decide(local_at(2026, 11, 2, 8, 0), &cfg);
        let line = d.status_line(&cfg);
        println!("status line: {}", line);
        assert!(line.contains("Day 90%"));
        let (next_at, _) = d.next.expect("next transition");
        assert!(line.contains(&format!("until {}", next_at.format("%H:%M"))));
        assert!(line.contains("Wrocław, Poland"));
    }
}
