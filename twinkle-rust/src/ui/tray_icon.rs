//! System tray icon using ksni (pure Rust StatusNotifierItem over D-Bus).
//!
//! GTK4 objects are not Send/Sync, so we use an async channel bridged
//! to the GLib main context. The ksni tray runs on a tokio background
//! task, and sends commands that are processed on the GTK main thread.

use crate::ui::brightness_popup::BrightnessPopup;
use gtk4::prelude::*;
use gtk4::{AboutDialog, Application, License};
use ksni::menu::StandardItem;
use ksni::{MenuItem, Tray, TrayMethods};
use std::sync::{Arc, Mutex};

/// Commands from the tray menu to the GTK4 main thread.
#[derive(Debug)]
pub enum TrayCommand {
    ShowBrightness,
    SetAllBrightness(u16),
    ShowSettings,
    ShowAbout,
    Quit,
}

/// The ksni Tray state. Only holds a channel sender — no GTK4 objects.
struct TrayState {
    tx: tokio::sync::mpsc::UnboundedSender<TrayCommand>,
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
            description: "Monitor brightness control".into(),
            ..Default::default()
        }
    }

    /// Left-click on tray icon opens brightness popup directly.
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.tx.send(TrayCommand::ShowBrightness);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx_show = self.tx.clone();
        let tx_10 = self.tx.clone();
        let tx_20 = self.tx.clone();
        let tx_40 = self.tx.clone();
        let tx_60 = self.tx.clone();
        let tx_80 = self.tx.clone();
        let tx_100 = self.tx.clone();
        let tx_settings = self.tx.clone();
        let tx_about = self.tx.clone();
        let tx_quit = self.tx.clone();

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
            // Quick brightness presets — set ALL monitors at once
            StandardItem {
                label: "  10%  (Night)".into(),
                icon_name: "weather-clear-night-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_10.send(TrayCommand::SetAllBrightness(10));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "  20%  (Dusk)".into(),
                icon_name: "weather-overcast-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_20.send(TrayCommand::SetAllBrightness(20));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "  40%  (Cloudy)".into(),
                icon_name: "weather-few-clouds-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_40.send(TrayCommand::SetAllBrightness(40));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "  60%  (Sunny)".into(),
                icon_name: "weather-clear-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_60.send(TrayCommand::SetAllBrightness(60));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "  80%  (Full Sun)".into(),
                icon_name: "weather-clear-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_80.send(TrayCommand::SetAllBrightness(80));
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "  100%  (Max)".into(),
                icon_name: "weather-clear-symbolic".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = tx_100.send(TrayCommand::SetAllBrightness(100));
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
    _handle: ksni::Handle<TrayState>,
}

impl TrayIcon {
    pub fn new(
        app: &Application,
        popup: Arc<Mutex<Option<BrightnessPopup>>>,
        config_manager: Arc<tokio::sync::Mutex<crate::core::ConfigManager>>,
        ddc_manager: Arc<crate::ddc::DDCManager>,
    ) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TrayCommand>();

        let tray_state = TrayState { tx };

        // Spawn the ksni tray service
        let handle = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async { tray_state.spawn().await })
        });

        let handle = match handle {
            Ok(h) => {
                tracing::info!("System tray icon created successfully");
                h
            }
            Err(e) => {
                tracing::error!("Failed to create system tray icon: {}", e);
                panic!("Failed to create system tray icon: {}", e);
            }
        };

        // Process tray commands on the GLib main context.
        let app_clone = app.clone();
        let popup_clone = popup.clone();
        let config_clone = config_manager.clone();
        let ddc_clone = ddc_manager.clone();

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
                        gtk4::glib::spawn_future_local(async move {
                            let monitors = ddc.get_monitors().await;
                            for m in &monitors {
                                if let Err(e) = ddc.set_brightness(&m.unique_id(), value).await {
                                    tracing::warn!(
                                        "Quick set failed on {}: {}",
                                        m.display_name(),
                                        e
                                    );
                                }
                            }
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
                        let dialog = AboutDialog::builder()
                            .program_name("Twinkle Linux")
                            .version(env!("CARGO_PKG_VERSION"))
                            .comments("Monitor brightness control for Linux")
                            .license_type(License::MitX11)
                            .website("https://github.com/ebarczynski/twinkle-linux")
                            .authors(vec!["Edwin Barczynski".to_string()])
                            .build();
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

        Self { _handle: handle }
    }
}
