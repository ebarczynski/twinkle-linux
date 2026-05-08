//! System tray icon implementation using libappindicator/Ayatana AppIndicator.

use crate::core::config::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::brightness_popup::BrightnessPopup;
use libappindicator::AppIndicator;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// Use GTK3 prelude for the tray menu (libappindicator requires GTK3 types).
use gtk3::prelude::*;

// Selective gtk4 imports — import only the traits we need to avoid conflict with gtk3::prelude::*.
use gtk4::Application;
use gtk4::ApplicationWindow;
use gtk4::AboutDialog;
use gtk4::License;
use gtk4::prelude::ApplicationExt;
use gtk4::prelude::GtkApplicationExt;
use gtk4::prelude::GtkWindowExt;
use gtk4::prelude::NativeDialogExt;
use gtk4::prelude::WidgetExt as Gtk4WidgetExt;

pub struct TrayIcon {
    indicator: std::sync::Mutex<AppIndicator>,
    ddc_manager: Arc<DDCManager>,
    config_manager: Arc<TokioMutex<ConfigManager>>,
}

impl TrayIcon {
    pub async fn new(
        app: Application,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<TokioMutex<ConfigManager>>,
        brightness_popup: Arc<std::sync::Mutex<Option<BrightnessPopup>>>,
        _window: ApplicationWindow,
    ) -> Self {
        let mut indicator = AppIndicator::new("Twinkle Linux", "display-brightness-symbolic");
        indicator.set_status(libappindicator::AppIndicatorStatus::Active);
        indicator.set_title("Twinkle Linux");

        let menu = gtk3::Menu::new();

        // Brightness Control
        let popup_ref = brightness_popup.clone();
        let item = gtk3::MenuItem::with_label("Brightness Control");
        item.connect_activate(move |_| {
            tracing::info!("Tray: Brightness Control clicked");
            if let Some(popup) = popup_ref.lock().unwrap().as_ref() {
                popup.popup();
            }
        });
        menu.add(&item);

        // Settings
        let config_ref = config_manager.clone();
        let app_ref = app.clone();
        let item = gtk3::MenuItem::with_label("Settings");
        item.connect_activate(move |_| {
            tracing::info!("Tray: Settings clicked");
            let config_mgr = config_ref.clone();
            let app_clone = app_ref.clone();
            gtk4::glib::spawn_future_local(async move {
                let windows = app_clone.windows();
                let parent = windows.first().cloned();
                if let Some(parent_win) = parent {
                    let dialog = crate::ui::widgets::settings_dialog::SettingsDialog::new(&parent_win, config_mgr).await;
                    dialog.run();
                }
            });
        });
        menu.add(&item);

        // About
        let app_ref = app.clone();
        let item = gtk3::MenuItem::with_label("About");
        item.connect_activate(move |_| {
            tracing::info!("Tray: About clicked");
            let about = AboutDialog::builder()
                .program_name("Twinkle Linux")
                .version(crate::utils::version())
                .comments("Control external monitor brightness via DDC/CI")
                .website("https://github.com/ebarczynski/twinkle-linux")
                .website_label("GitHub Repository")
                .license_type(License::MitX11)
                .authors(vec!["Edwin Barczynski"])
                .build();
            let windows = app_ref.windows();
            about.set_transient_for(windows.first());
            about.present();
        });
        menu.add(&item);

        let sep = gtk3::SeparatorMenuItem::new();
        menu.add(&sep);

        // Quit
        let app_ref = app.clone();
        let item = gtk3::MenuItem::with_label("Quit");
        item.connect_activate(move |_| {
            tracing::info!("Tray: Quit clicked");
            app_ref.quit();
        });
        menu.add(&item);

        menu.show_all();
        indicator.set_menu(&mut { let m = menu; m });

        Self {
            indicator: std::sync::Mutex::new(indicator),
            ddc_manager,
            config_manager,
        }
    }

    pub async fn update_state(&self) {
        let monitors = self.ddc_manager.get_monitors().await;
        let mut indicator = self.indicator.lock().unwrap();
        if monitors.is_empty() {
            indicator.set_icon("dialog-warning-symbolic");
            indicator.set_title("Twinkle Linux - No monitors detected");
        } else {
            indicator.set_icon("display-brightness-symbolic");
            let title = format!("Twinkle Linux - {} monitor(s)", monitors.len());
            indicator.set_title(&title);
        }
    }
}
