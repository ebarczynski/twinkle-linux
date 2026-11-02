//! Debug helper: print the current day/night schedule decision.
//!
//! Usage: `cargo run --example schedule_check` (optionally with a config
//! file present at ~/.config/twinkle-linux/config.json).

use twinkle_linux::core::config::{ConfigManager, ScheduleConfig};
use twinkle_linux::core::schedule;

fn main() {
    // Try the user's config for the schedule parameters; fall back to
    // compiled-in defaults.
    let cfg = match ConfigManager::new() {
        Ok(mut mgr) => {
            if mgr.load().is_ok() {
                mgr.config().schedule.clone()
            } else {
                ScheduleConfig::default()
            }
        }
        Err(_) => ScheduleConfig::default(),
    };

    let now = chrono::Local::now();
    println!("Location   : {}", cfg.location_name);
    println!("Coordinates: {:.4}, {:.4}", cfg.latitude, cfg.longitude);
    println!("Now (local): {}", now.format("%Y-%m-%d %H:%M:%S %Z"));

    let today = now.date_naive();
    let offset = now.offset().to_owned();
    if let Some((sunrise, sunset)) =
        schedule::sun_times_local(today, offset.local_minus_utc(), cfg.latitude, cfg.longitude)
    {
        println!("Sunrise    : {}", sunrise.format("%H:%M:%S %Z"));
        println!("Sunset     : {}", sunset.format("%H:%M:%S %Z"));
    } else {
        println!("Sunrise    : (polar day/night — no events today)");
    }

    let decision = schedule::decide(now, &cfg);
    println!("Phase      : {}", decision.phase.name());
    println!("Target     : {}%", decision.target);
    println!("Ramping    : {}", decision.ramping);
    match &decision.next {
        Some((at, to)) => println!(
            "Next       : {} {} at {}",
            to.name(),
            to.brightness(&cfg),
            at.format("%H:%M")
        ),
        None => println!("Next       : none"),
    }
    println!("Status line: {}", decision.status_line(&cfg));
}
