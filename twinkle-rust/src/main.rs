//! Twinkle Linux - GUI application for controlling external monitor brightness via DDC/CI on Linux.

mod core;
mod ddc;
mod ui;
mod utils;

use crate::core::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::brightness_popup::BrightnessPopup;
use crate::ui::tray_icon::TrayIcon;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
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

    // Create a hidden window. Needed as a transient parent for popups/dialogs,
    // but never shown to the user — this is a tray-only app.
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Twinkle Linux")
        .default_width(1)
        .default_height(1)
        .build();
    // Do NOT call window.show()

    let brightness_popup: Arc<std::sync::Mutex<Option<BrightnessPopup>>> =
        Arc::new(std::sync::Mutex::new(None));

    // Clone for async context
    let ddc_manager_for_init = state.ddc_manager.clone();
    let config_manager_for_init = state.config_manager.clone();
    let initialized_for_async = state.initialized.clone();
    let app_clone = app.clone();
    let brightness_popup_clone = brightness_popup.clone();
    let window_clone = window.clone();

    // Spawn async initialization on the GLib main context.
    gtk4::glib::spawn_future_local(async move {
        tracing::info!("Async initialization task started");

        // Initialize DDC manager first (detect monitors)
        tracing::info!("Initializing DDC manager...");
        let init_ok = match ddc_manager_for_init.initialize().await {
            Ok(true) => {
                tracing::info!("DDC manager initialized successfully");
                true
            }
            Ok(false) => {
                tracing::warn!("No monitors detected");
                false
            }
            Err(e) => {
                tracing::error!("DDC init error: {}", e);
                false
            }
        };

        // Create brightness popup
        tracing::info!("Creating BrightnessPopup...");
        let popup = BrightnessPopup::new(
            &window_clone,
            ddc_manager_for_init.clone(),
            config_manager_for_init.clone(),
        )
        .await;
        if init_ok {
            popup.refresh_monitors().await;
        }
        *brightness_popup_clone.lock().unwrap() = Some(popup);
        tracing::info!("BrightnessPopup created");

        // Create tray icon — it appears in the system tray immediately.
        // Pass references to popup and window so menu items work.
        tracing::info!("Creating TrayIcon...");
        let tray = TrayIcon::new(
            app_clone.clone(),
            ddc_manager_for_init.clone(),
            config_manager_for_init.clone(),
            brightness_popup_clone.clone(),
            window_clone,
        )
        .await;

        // Update tray icon state
        if init_ok {
            tray.update_state().await;
        }

        tracing::info!("TrayIcon created, tray icon should now be visible");

        // Keep tray icon alive for the rest of the application lifetime.
        // It will be dropped when the app shuts down.
        // Leak it intentionally — it must outlive this async block.
        std::mem::forget(tray);

        initialized_for_async.store(true, Ordering::SeqCst);
        tracing::info!("Initialization complete");
    });

    tracing::info!("build_ui completed");
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

    // Enter the tokio runtime context on the main thread.
    // This guard lives for the entire application lifetime, so ALL async
    // code running on the GLib main context (via glib::spawn_future_local)
    // will find a valid tokio reactor.
    let _rt_guard = runtime.enter();
    let _runtime = runtime;

    // Initialize GTK application
    let app = Application::builder()
        .application_id("com.github.ebarczynski.TwinkleLinux")
        .build();

    app.connect_activate(move |app| {
        tracing::info!("App activate callback started");

        let rt = tokio::runtime::Handle::current();

        // Create config manager and load config
        tracing::info!("Creating ConfigManager...");
        let config_manager = rt.block_on(async {
            let mut mgr = ConfigManager::new().expect("Failed to create config manager");
            if let Err(e) = mgr.load() {
                tracing::warn!("Failed to load config, using defaults: {}", e);
            }
            mgr
        });

        // Create DDC manager
        tracing::info!("Creating DDCManager...");
        let ddc_manager = Arc::new(
            rt.block_on(DDCManager::new())
                .expect("Failed to create DDC manager"),
        );

        let state = AppState {
            ddc_manager,
            config_manager: Arc::new(Mutex::new(config_manager)),
            initialized: Arc::new(AtomicBool::new(false)),
        };

        build_ui(app, state);
    });

    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    tracing::info!("Twinkle Linux shutdown complete");
}
