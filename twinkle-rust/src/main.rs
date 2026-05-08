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
use tokio::sync::Mutex as TokioMutex;

struct AppState {
    ddc_manager: Arc<DDCManager>,
    config_manager: Arc<TokioMutex<ConfigManager>>,
    initialized: Arc<AtomicBool>,
}

fn build_ui(app: &Application, state: AppState) {
    tracing::info!("build_ui: Starting UI construction");

    // Create a hidden window. This serves as the toplevel parent for popovers
    // and dialogs. It must exist and be realized for GTK4 popover parenting.
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Twinkle Linux")
        .default_width(1)
        .default_height(1)
        .decorated(false)
        .build();

    // Realize the window so it gets a GdkSurface, but keep it invisible.
    // A zero-size undecorated window won't be visible to the user.
    // We do NOT call window.show() or window.present().
    // However, GTK4 requires the widget to be realized for popover parenting.
    // We can't realize without showing in GTK4 easily, so we use a different
    // approach: the popover will be parented to the window, and we show/hide
    // the window along with the popover as needed.

    let popup_arc: Arc<std::sync::Mutex<Option<BrightnessPopup>>> =
        Arc::new(std::sync::Mutex::new(None));

    let ddc_manager = state.ddc_manager.clone();
    let config_manager = state.config_manager.clone();
    let initialized = state.initialized.clone();
    let app_clone = app.clone();
    let window_clone = window.clone();

    gtk4::glib::spawn_future_local(async move {
        tracing::info!("Async initialization task started");

        tracing::info!("Initializing DDC manager...");
        let _init_ok = match ddc_manager.initialize().await {
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

        tracing::info!("Creating BrightnessPopup...");
        let popup = BrightnessPopup::new(
            &window_clone,
            ddc_manager.clone(),
            config_manager.clone(),
        )
        .await;
        // No separate refresh_monitors() call — popup() rebuilds cards each time
        *popup_arc.lock().unwrap() = Some(popup);
        tracing::info!("BrightnessPopup created");

        tracing::info!("Creating TrayIcon...");
        let _tray = TrayIcon::new(&app_clone, popup_arc.clone(), config_manager.clone(), ddc_manager.clone());
        tracing::info!("TrayIcon created, icon should now be visible in system tray");

        // Keep tray alive for the app lifetime
        std::mem::forget(_tray);

        initialized.store(true, Ordering::SeqCst);
        tracing::info!("Initialization complete");
    });

    tracing::info!("build_ui completed");
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Twinkle Linux v{}", utils::version());

    let _rt_guard = runtime.enter();
    let _runtime = runtime;

    let app = Application::builder()
        .application_id("com.github.ebarczynski.TwinkleLinux")
        .build();

    app.connect_activate(move |app| {
        tracing::info!("App activate callback started");

        tracing::info!("Creating ConfigManager...");
        let config_manager = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut mgr = ConfigManager::new().expect("Failed to create config manager");
                if let Err(e) = mgr.load() {
                    tracing::warn!("Failed to load config, using defaults: {}", e);
                }
                mgr
            })
        });

        tracing::info!("Creating DDCManager...");
        let ddc_manager = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(DDCManager::new())
            })
            .expect("Failed to create DDC manager"),
        );

        let state = AppState {
            ddc_manager,
            config_manager: Arc::new(TokioMutex::new(config_manager)),
            initialized: Arc::new(AtomicBool::new(false)),
        };

        build_ui(app, state);
    });

    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    tracing::info!("Twinkle Linux shutdown complete");
}
