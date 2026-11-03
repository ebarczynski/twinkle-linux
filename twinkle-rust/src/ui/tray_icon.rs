//! System tray icon using ksni (pure Rust StatusNotifierItem over D-Bus).
//!
//! GTK4 objects are not Send/Sync, so we use an async channel bridged
//! to the GLib main context. The ksni tray runs on a tokio background
//! task, and sends commands that are processed on the GTK main thread.
//!
//! The tray menu carries a brightness value LIST generated from the config
//! (`ui.preset_values` + `preset_labels`) — the same values the sliders in
//! the brightness details panel snap to. Selecting a value sets all
//! monitors. Keep menu items PLAIN (StandardItem + separators, constant
//! structure): exotic item types or dynamic row counts render as empty
//! menus on some shell AppIndicator implementations.

use crate::ui::brightness_popup::BrightnessPopup;
use gtk4::prelude::*;
use gtk4::{AboutDialog, Application, License};
use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray, TrayMethods};
use std::sync::{Arc, Mutex};

/// Project contact information (GitHub handle / repository).
pub const REPO_URL: &str = "https://github.com/ebarczynski/twinkle-linux";
pub const REPO_LABEL: &str = "github.com/ebarczynski/twinkle-linux";
pub const AUTHOR: &str = "Edwin Barczynski (@ebarczynski)";
pub const AUTHOR_EMAIL: &str = "barczynski.e@gmail.com";

/// Commands from the tray menu to the GTK4 main thread.
#[derive(Debug)]
pub enum TrayCommand {
    /// Show the full per-monitor brightness panel.
    ShowBrightness,
    /// Set ALL monitors to a configured preset value.
    SetAllBrightness(u16),
    /// Flip `schedule.enabled` in the config and refresh the menu.
    ToggleSchedule,
    /// Re-read the config file from disk and refresh the menu.
    ReloadConfig,
    ShowSettings,
    ShowAbout,
    Quit,
}

/// The ksni Tray state. Only holds a channel sender, the config-driven
/// preset list and cheap status snapshots — no GTK4 objects.
pub struct TrayState {
    tx: tokio::sync::mpsc::UnboundedSender<TrayCommand>,
    /// Brightness presets for the menu: (value, display label), sorted by
    /// value, derived from `ui.preset_values` + `preset_labels`.
    pub presets: Vec<(u16, String)>,
    /// Whether the day/night schedule is currently enabled.
    pub schedule_enabled: bool,
    /// One-line schedule status shown in the schedule item label, updated
    /// by the schedule engine, e.g. "Day 90% — until 16:26 Evening (Wrocław)".
    pub schedule_status: String,
}

/// Weather-ish icon for a brightness percentage (0 = night, 100 = full sun).
fn weather_icon(value: u16) -> &'static str {
    match value {
        0..=25 => "weather-clear-night-symbolic",
        26..=50 => "weather-overcast-symbolic",
        51..=75 => "weather-few-clouds-symbolic",
        _ => "weather-clear-symbolic",
    }
}

/// Menu label for a preset: "  10%  (Night)" or "  30%" when unlabeled.
fn preset_menu_label(value: u16, label: &str) -> String {
    if label.is_empty() {
        format!("  {}%", value)
    } else {
        format!("  {}%  ({})", value, label)
    }
}

impl Tray for TrayState {
    fn id(&self) -> String {
        "twinkle-linux".into()
    }

    fn title(&self) -> String {
        "Twinkle Linux".into()
    }

    fn status(&self) -> ksni::Status {
        ksni::Status::Active
    }

    fn icon_name(&self) -> String {
        "display-brightness".into()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: "Twinkle Linux".into(),
            description: format!("Monitor brightness control\n{}", REPO_LABEL),
            ..Default::default()
        }
    }

    /// Left-click on tray icon opens the brightness panel.
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayCommand::ShowBrightness);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        // NOTE: keep this menu PLAIN — only StandardItems and separators, with
        // a constant item structure (same count and order every time). Exotic
        // item types (checkmarks, disabled info rows) and dynamic row counts
        // are the classic ways shell AppIndicator implementations (GNOME
        // extension, MATE, ...) render an EMPTY menu — items turn invisible
        // and clicks stop matching. State lives in the label text instead.
        let tx_show = self.tx.clone();
        let tx_schedule = self.tx.clone();
        let tx_reload = self.tx.clone();
        let tx_settings = self.tx.clone();
        let tx_about = self.tx.clone();
        let tx_quit = self.tx.clone();

        let schedule_label = if self.schedule_enabled {
            if self.schedule_status.is_empty() {
                "Day/night schedule: ON".to_string()
            } else {
                format!("Schedule: ON — {}", self.schedule_status)
            }
        } else {
            "Day/night schedule: OFF".to_string()
        };

        let mut menu = vec![
            StandardItem {
                label: "Brightness details (per monitor)".into(),
                icon_name: "display-brightness-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_show.send(TrayCommand::ShowBrightness);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
        ];

        // Brightness presets from the config — the same values the sliders
        // snap to in the details panel.
        for (value, label) in &self.presets {
            let tx_preset = self.tx.clone();
            let value = *value;
            menu.push(
                StandardItem {
                    label: preset_menu_label(value, label),
                    icon_name: weather_icon(value).into(),
                    activate: Box::new(move |_this: &mut Self| {
                        let _ = tx_preset.send(TrayCommand::SetAllBrightness(value));
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        menu.push(MenuItem::Separator);
        // Single toggle item: click flips schedule.enabled. ON shows the
        // live status inline (e.g. "Schedule: ON — Day 90% — until 16:26
        // Evening (Wrocław, Poland)").
        menu.push(
            StandardItem {
                label: schedule_label,
                icon_name: "day-night-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_schedule.send(TrayCommand::ToggleSchedule);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(
            StandardItem {
                label: "Reload config".into(),
                icon_name: "view-refresh-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_reload.send(TrayCommand::ReloadConfig);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(MenuItem::Separator);
        menu.push(
            StandardItem {
                label: "Settings".into(),
                icon_name: "preferences-system-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_settings.send(TrayCommand::ShowSettings);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(
            StandardItem {
                label: "About".into(),
                icon_name: "help-about-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_about.send(TrayCommand::ShowAbout);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu.push(MenuItem::Separator);
        menu.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_quit.send(TrayCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        );
        menu
    }
}

pub struct TrayIcon {
    handle: Option<Arc<ksni::Handle<TrayState>>>,
}

impl TrayIcon {
    pub fn new(
        app: &Application,
        popup: Arc<Mutex<Option<BrightnessPopup>>>,
        config_manager: Arc<tokio::sync::Mutex<crate::core::ConfigManager>>,
        ddc_manager: Arc<crate::ddc::DDCManager>,
        manual_tx: Option<tokio::sync::mpsc::UnboundedSender<crate::core::schedule::ScheduleEvent>>,
    ) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TrayCommand>();

        // Initial preset list + schedule state from the config.
        let (presets, schedule_enabled) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let cfg = config_manager.lock().await;
                let config = cfg.config();
                (
                    crate::utils::preset_menu_entries(
                        &config.ui.preset_values,
                        &config.ui.preset_labels,
                        config.behavior.min_brightness,
                    ),
                    config.schedule.enabled,
                )
            })
        });

        let tray_state = TrayState {
            tx,
            presets,
            schedule_enabled,
            schedule_status: String::new(),
        };

        // Spawn the ksni tray service.
        //
        // Non-fatal: a missing StatusNotifierWatcher host (bare Xvfb, unusual
        // DEs, CI) previously PANICKED the whole app after the popup was
        // already working. Log loudly and continue without a tray instead.
        let handle = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { tray_state.spawn().await })
        }) {
            Ok(h) => {
                tracing::info!("System tray icon created successfully");
                Some(Arc::new(h))
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create system tray icon: {} — continuing WITHOUT tray. \
                     Set TWINKLE_AUTOSHOW=1 to open the brightness panel at startup.",
                    e
                );
                None
            }
        };
        let handle_for_commands = handle.clone();

        // Process tray commands on the GLib main context.
        let app_clone = app.clone();
        let popup_clone = popup.clone();
        let config_clone = config_manager.clone();
        let ddc_clone = ddc_manager.clone();
        let manual_tx_clone = manual_tx.clone();

        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Drain all pending commands
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    TrayCommand::ShowBrightness => {
                        if let Some(ref popup) = *popup_clone.lock().unwrap() {
                            popup.popup();
                        }
                    }
                    TrayCommand::SetAllBrightness(value) => {
                        let ddc = ddc_clone.clone();
                        let config = config_clone.clone();
                        let manual_tx = manual_tx_clone.clone();
                        gtk4::glib::spawn_future_local(async move {
                            let min = {
                                let cfg = config.lock().await;
                                cfg.config().behavior.min_brightness
                            };
                            let clamped = value.max(min);
                            let monitors = ddc.get_monitors().await;
                            for m in &monitors {
                                if let Err(e) = ddc.set_brightness(&m.unique_id(), clamped).await {
                                    tracing::warn!(
                                        "Preset set failed on {}: {}",
                                        m.display_name(),
                                        e
                                    );
                                }
                            }
                            // Manual change suspends the day/night schedule
                            // for the current phase (same as slider drags).
                            if let Some(tx) = manual_tx {
                                let _ = tx.send(crate::core::schedule::ScheduleEvent::ManualChange);
                            }
                        });
                    }
                    TrayCommand::ToggleSchedule => {
                        let config = config_clone.clone();
                        let handle = handle_for_commands.clone();
                        gtk4::glib::spawn_future_local(async move {
                            let enabled = {
                                let mut mgr = config.lock().await;
                                mgr.config_mut().schedule.enabled = !mgr.config().schedule.enabled;
                                let enabled = mgr.config().schedule.enabled;
                                if let Err(e) = mgr.save() {
                                    tracing::error!("Failed to save schedule toggle: {}", e);
                                }
                                enabled
                            };
                            if let Some(handle) = handle.as_ref() {
                                handle
                                    .update(|t| {
                                        t.schedule_enabled = enabled;
                                        if !enabled {
                                            t.schedule_status = "Auto schedule off".into();
                                        }
                                    })
                                    .await;
                            }
                            tracing::info!("Day/night schedule toggled: {}", enabled);
                        });
                    }
                    TrayCommand::ReloadConfig => {
                        let config = config_clone.clone();
                        let handle = handle_for_commands.clone();
                        gtk4::glib::spawn_future_local(async move {
                            let (enabled, presets) = {
                                let mut mgr = config.lock().await;
                                if let Err(e) = mgr.load() {
                                    tracing::warn!("Failed to reload config: {}", e);
                                }
                                let config = mgr.config();
                                (
                                    config.schedule.enabled,
                                    crate::utils::preset_menu_entries(
                                        &config.ui.preset_values,
                                        &config.ui.preset_labels,
                                        config.behavior.min_brightness,
                                    ),
                                )
                            };
                            if let Some(handle) = handle.as_ref() {
                                handle
                                    .update(|t| {
                                        t.schedule_enabled = enabled;
                                        t.presets = presets;
                                        if !enabled {
                                            t.schedule_status = "Auto schedule off".into();
                                        }
                                    })
                                    .await;
                            }
                            tracing::info!("Config reloaded from disk");
                        });
                    }
                    TrayCommand::ShowSettings => {
                        let windows = app_clone.windows();
                        if let Some(parent) = windows.first() {
                            let parent = parent.clone();
                            gtk4::glib::spawn_future_local({
                                let config = config_clone.clone();
                                async move {
                                    let dialog =
                                        crate::ui::widgets::settings_dialog::SettingsDialog::new(
                                            &parent, config,
                                        )
                                        .await;
                                    dialog.present();
                                }
                            });
                        }
                    }
                    TrayCommand::ShowAbout => {
                        let windows = app_clone.windows();
                        let parent = windows.first().cloned();
                        let contact = format!("{} <{}>", AUTHOR, AUTHOR_EMAIL);
                        let dialog = AboutDialog::builder()
                            .program_name("Twinkle Linux")
                            .version(env!("CARGO_PKG_VERSION"))
                            .comments(format!(
                                "Monitor brightness control for Linux\n\n{}",
                                REPO_LABEL
                            ))
                            .license_type(License::MitX11)
                            .website(REPO_URL)
                            .website_label(REPO_LABEL)
                            .build();
                        dialog.set_authors(&[AUTHOR]);
                        dialog.add_credit_section("Contact", &[contact.as_str()]);
                        if let Some(p) = parent {
                            dialog.set_transient_for(Some(&p));
                        }
                        dialog.present();
                    }
                    TrayCommand::Quit => {
                        app_clone.quit();
                    }
                }
            }
            gtk4::glib::ControlFlow::Continue
        });

        Self { handle }
    }

    /// Shared tray handle — the schedule engine uses it to push live
    /// status updates into the menu. `None` when no tray host is available.
    pub fn tray_handle(&self) -> Option<Arc<ksni::Handle<TrayState>>> {
        self.handle.clone()
    }
}
