//! Twinkle Linux - GUI application for controlling external monitor brightness via DDC/CI on Linux.

mod core;
mod ddc;
mod ui;
mod utils;

use crate::core::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::brightness_popup::BrightnessPopup;
use crate::ui::tray_icon::TrayIcon;
use crate::ui::widgets::settings_dialog::SettingsDialog;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Application, ApplicationWindow, Box, Label, Orientation,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

/// Application state shared across UI components.
struct AppState {
    /// DDC manager for monitor communication
    ddc_manager: Arc<DDCManager>,
    /// Config manager for persistent settings
    config_manager: Arc<Mutex<ConfigManager>>,
    /// Whether the app is fully initialized
    initialized: Arc<AtomicBool>,
}

/// Build the application UI.
fn build_ui(app: &Application, state: AppState) {
    tracing::info!("build_ui: Starting UI construction");

    // Create the tray icon (system tray integration)
    let tray_icon = Arc::new(std::sync::Mutex::new(None::<TrayIcon>));
    let brightness_popup = Arc::new(std::sync::Mutex::new(None::<BrightnessPopup>));

    // Create main window (hidden by default, shown via tray icon)
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Twinkle Linux")
        .default_width(400)
        .default_height(300)
        .build();

    // Create main container
    let container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Create status label
    let status_label = Label::builder()
        .label("Initializing...")
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    container.append(&status_label);
    window.set_child(Some(&container));

    // Clone for async context
    let ddc_manager_for_init = state.ddc_manager.clone();
    let config_manager_for_init = state.config_manager.clone();
    let initialized_for_async = state.initialized.clone();
    let app_clone = app.clone();
    let tray_icon_clone = tray_icon.clone();
    let brightness_popup_clone = brightness_popup.clone();
    let window_clone = window.clone();

    // Spawn async initialization
    glib::spawn_future_local(async move {
        tracing::info!("Async initialization task started");

        // Create and set up the tray icon
        tracing::info!("Creating TrayIcon...");
        let tray = TrayIcon::new(
            app_clone.clone(),
            ddc_manager_for_init.clone(),
            config_manager_for_init.clone(),
        )
        .await;
        tracing::info!("TrayIcon created successfully");

        // Store tray icon
        *tray_icon_clone.lock().unwrap() = Some(tray);

        // Initialize DDC manager (detect monitors)
        tracing::info!("Initializing DDC manager...");
        match ddc_manager_for_init.initialize().await {
            Ok(true) => {
                tracing::info!("DDC manager initialized successfully");

                // Update tray icon state
                if let Some(tray) = tray_icon_clone.lock().unwrap().as_ref() {
                    tray.update_state().await;
                }

                // Create brightness popup
                tracing::info!("Creating BrightnessPopup...");
                let popup = BrightnessPopup::new(
                    &window_clone,
                    ddc_manager_for_init.clone(),
                    config_manager_for_init.clone(),
                )
                .await;
                popup.refresh_monitors().await;
                *brightness_popup_clone.lock().unwrap() = Some(popup);

                // Mark as initialized
                initialized_for_async.store(true, Ordering::SeqCst);
                tracing::info!("Initialization complete");

                // Hide the window (app runs as tray icon)
                window_clone.hide();
            }
            Ok(false) => {
                tracing::warn!("DDC manager initialization failed - no monitors detected");

                // Still create popup (empty) and mark initialized so UI is usable
                let popup = BrightnessPopup::new(
                    &window_clone,
                    ddc_manager_for_init.clone(),
                    config_manager_for_init.clone(),
                )
                .await;
                *brightness_popup_clone.lock().unwrap() = Some(popup);
                initialized_for_async.store(true, Ordering::SeqCst);

                // Update status label
                window_clone.set_title(Some("Twinkle Linux - No Monitors"));
            }
            Err(e) => {
                tracing::error!("DDC manager initialization error: {}", e);

                // Still mark initialized so the app doesn't hang
                initialized_for_async.store(true, Ordering::SeqCst);
            }
        }
        tracing::info!("Async initialization task completed");
    });

    // Setup tray actions with references to popup and settings
    setup_tray_actions(app, tray_icon, brightness_popup, state);

    window.show();
    tracing::info!("build_ui: Window shown");
}

/// Create the application actions for the tray icon menu.
fn setup_tray_actions(
    app: &Application,
    tray_icon: Arc<std::sync::Mutex<Option<TrayIcon>>>,
    brightness_popup: Arc<std::sync::Mutex<Option<BrightnessPopup>>>,
    state: AppState,
) {
    // Show brightness popup action
    let show_brightness = gtk4::gio::SimpleAction::new("show-brightness", None);
    let popup_for_brightness = brightness_popup.clone();
    show_brightness.connect_activate(move |_, _| {
        tracing::info!("Show brightness popup");
        if let Some(popup) = popup_for_brightness.lock().unwrap().as_ref() {
            popup.popup();
        } else {
            tracing::warn!("Brightness popup not yet initialized");
        }
    });
    app.add_action(&show_brightness);

    // Show settings dialog action
    let show_settings = gtk4::gio::SimpleAction::new("show-settings", None);
    let config_for_settings = state.config_manager.clone();
    let window_for_settings = app.clone();
    show_settings.connect_activate(move |_, _| {
        tracing::info!("Show settings dialog");
        let config_mgr = config_for_settings.clone();
        let app_ref = window_for_settings.clone();

        glib::spawn_future_local(async move {
            // Get or create a transient window for the dialog
            let windows = app_ref.windows();
            let parent = windows.first().cloned();

            if let Some(parent_win) = parent {
                let dialog = SettingsDialog::new(&parent_win, config_mgr).await;
                dialog.run();
            }
        });
    });
    app.add_action(&show_settings);

    // Show about dialog action
    let show_about = gtk4::gio::SimpleAction::new("show-about", None);
    let app_for_about = app.clone();
    show_about.connect_activate(move |_, _| {
        tracing::info!("Show about dialog");

        let about = AboutDialog::builder()
            .program_name("Twinkle Linux")
            .version(utils::version())
            .comments("Control external monitor brightness via DDC/CI")
            .website("https://github.com/ebarczynski/twinkle-linux")
            .website_label("GitHub Repository")
            .license_type(gtk4::License::MitX11)
            .authors(vec!["Edwin Barczynski"])
            .build();

        about.set_transient_for(app_for_about.windows().first());
        about.present();
    });
    app.add_action(&show_about);

    // Quit action
    let quit = gtk4::gio::SimpleAction::new("quit", None);
    let app_clone = app.clone();
    quit.connect_activate(move |_, _| {
        tracing::info!("Quit action triggered");
        app_clone.quit();
    });
    app.add_action(&quit);
}

/// Main entry point.
fn main() {
    // Create tokio runtime for async operations
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Twinkle Linux v{}", utils::version());

    // Get a handle to the runtime before moving it
    let rt_handle = runtime.handle().clone();
    // Keep the runtime alive for the duration of the application
    let _runtime = runtime;

    // Initialize GTK application
    let app = Application::builder()
        .application_id("com.github.ebarczynski.TwinkleLinux")
        .build();

    // Connect activate signal
    app.connect_activate(move |app| {
        tracing::info!("App activate callback started");

        // Create config manager and load existing config
        tracing::info!("Creating ConfigManager...");
        let config_manager = rt_handle.block_on(async {
            let mut mgr = ConfigManager::new().expect("Failed to create config manager");
            if let Err(e) = mgr.load() {
                tracing::warn!("Failed to load config, using defaults: {}", e);
            }
            mgr
        });
        tracing::info!("ConfigManager created and config loaded");

        // Create DDC manager
        tracing::info!("Creating DDCManager...");
        let ddc_manager = Arc::new(
            rt_handle.block_on(DDCManager::new())
                .expect("Failed to create DDC manager"),
        );
        tracing::info!("DDCManager created successfully");

        let state = AppState {
            ddc_manager,
            config_manager: Arc::new(Mutex::new(config_manager)),
            initialized: Arc::new(AtomicBool::new(false)),
        };

        tracing::info!("Calling build_ui...");
        build_ui(app, state);
        tracing::info!("build_ui completed");
    });

    // Run the GTK application (blocks until quit)
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    tracing::info!("Twinkle Linux shutdown complete");
}
