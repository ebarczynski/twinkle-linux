//! System tray icon using ksni (pure Rust StatusNotifierItem over D-Bus).
//!
//! GTK4 objects are not Send/Sync, so we use an async channel bridged
//! to the GLib main context. The ksni tray runs on a tokio background
//! task, and sends commands that are processed on the GTK main thread.
//!
//! The tray menu intentionally contains NO brightness value list — the
//! slider popup (left click / "Brightness Control") is the brightness
//! control, snapping to the values configured in `ui.preset_values`.

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
    ShowBrightness,
    /// Flip `schedule.enabled` in the config and refresh the menu.
    ToggleSchedule,
    /// Re-read the config file from disk and refresh the menu.
    ReloadConfig,
    ShowSettings,
    ShowAbout,
    Quit,
}

/// The ksni Tray state. Only holds a channel sender and cheap status
/// snapshots — no GTK4 objects.
pub struct TrayState {
    tx: tokio::sync::mpsc::UnboundedSender<TrayCommand>,
    /// Whether the day/night schedule is currently enabled.
    pub schedule_enabled: bool,
    /// One-line schedule status shown (disabled) in the menu, updated by
    /// the schedule engine, e.g. "Day 90% — until 16:24 Evening (Wrocław)".
    pub schedule_status: String,
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

    /// Left-click on tray icon opens brightness popup directly.
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

        vec![
            StandardItem {
                label: "Brightness Control".into(),
                icon_name: "display-brightness-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_show.send(TrayCommand::ShowBrightness);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            // Single toggle item: click flips schedule.enabled. ON shows the
            // live status inline (e.g. "Schedule: ON — Day 90% — until 16:26
            // Evening (Wrocław, Poland)").
            StandardItem {
                label: schedule_label,
                icon_name: "day-night-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_schedule.send(TrayCommand::ToggleSchedule);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Reload config".into(),
                icon_name: "view-refresh-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_reload.send(TrayCommand::ReloadConfig);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Settings".into(),
                icon_name: "preferences-system-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_settings.send(TrayCommand::ShowSettings);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "About".into(),
                icon_name: "help-about-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_about.send(TrayCommand::ShowAbout);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_quit.send(TrayCommand::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
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
    ) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TrayCommand>();

        let tray_state = TrayState {
            tx,
            schedule_enabled: false,
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
                     Set TWINKLE_AUTOSHOW=1 to open the popup at startup.",
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

        gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            // Drain all pending commands
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    TrayCommand::ShowBrightness => {
                        if let Some(ref popup) = *popup_clone.lock().unwrap() {
                            popup.popup();
                        }
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
                            let enabled = {
                                let mut mgr = config.lock().await;
                                if let Err(e) = mgr.load() {
                                    tracing::warn!("Failed to reload config: {}", e);
                                }
                                mgr.config().schedule.enabled
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
