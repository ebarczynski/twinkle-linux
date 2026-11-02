//! Twinkle Linux - GUI application for controlling external monitor brightness via DDC/CI on Linux.

#![allow(dead_code)]

mod core;
mod ddc;
mod ui;
mod utils;

use crate::core::schedule::ScheduleEvent;
use crate::core::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::brightness_popup::BrightnessPopup;
use crate::ui::tray_icon::TrayIcon;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

struct AppState {
    ddc_manager: Arc<DDCManager>,
    config_manager: Arc<TokioMutex<ConfigManager>>,
    initialized: Arc<AtomicBool>,
}

fn build_ui(app: &Application, state: AppState) {
    tracing::info!("build_ui: Starting UI construction");

    // Apply the configured theme before any widget is realized (see
    // `ui::apply_theme_preference` for why native widgets need this).
    crate::ui::apply_theme_preference(
        &state
            .config_manager
            .blocking_lock()
            .config()
            .general
            .theme
            .clone(),
    );

    // Create a hidden window. This serves as the toplevel parent for popovers
    // and dialogs. It must exist and be realized for GTK4 popover parenting.
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Twinkle Linux")
        .default_width(1)
        .default_height(1)
        .decorated(false)
        .build();

    #[allow(clippy::arc_with_non_send_sync)]
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
        // Manual-change channel: popup → schedule engine.
        let (manual_tx, manual_rx) = tokio::sync::mpsc::unbounded_channel::<ScheduleEvent>();
        let popup = BrightnessPopup::new(
            &window_clone,
            ddc_manager.clone(),
            config_manager.clone(),
            Some(manual_tx),
        )
        .await;
        *popup_arc.lock().unwrap() = Some(popup);
        tracing::info!("BrightnessPopup created");

        // TWINKLE_AUTOSHOW=1: open the popup at startup. Debugging aid that
        // also makes the app usable on systems without a tray host.
        if std::env::var("TWINKLE_AUTOSHOW").as_deref() == Ok("1") {
            if let Some(ref popup) = *popup_arc.lock().unwrap() {
                popup.popup();
            }
        }

        tracing::info!("Creating TrayIcon...");
        let tray = TrayIcon::new(&app_clone, popup_arc.clone(), config_manager.clone());
        tracing::info!("TrayIcon created, icon should now be visible in system tray");

        // Spawn the day/night schedule engine (sunrise/sunset cycling).
        let engine = crate::core::schedule::spawn_engine(
            ddc_manager.clone(),
            config_manager.clone(),
            tray.tray_handle(),
            manual_rx,
        );
        // Detach: the engine runs for the app lifetime.
        drop(engine);

        // Restore last brightness on startup (unless the schedule will
        // immediately set its own target — avoid fighting it).
        {
            let (restore, schedule_enabled, min_brightness) = {
                let cfg = config_manager.lock().await;
                (
                    cfg.config().behavior.restore_brightness,
                    cfg.config().schedule.enabled,
                    cfg.config().behavior.min_brightness,
                )
            };
            if restore && !schedule_enabled {
                let monitors = ddc_manager.get_monitors().await;
                for m in &monitors {
                    let last = {
                        let cfg = config_manager.lock().await;
                        cfg.config()
                            .monitors
                            .iter()
                            .find(|mc| mc.unique_id == m.unique_id())
                            .map(|mc| mc.last_brightness)
                    };
                    if let Some(b) = last {
                        if let Err(e) = ddc_manager
                            .set_brightness(&m.unique_id(), b.max(min_brightness))
                            .await
                        {
                            tracing::warn!(
                                "Restore brightness failed on {}: {}",
                                m.display_name(),
                                e
                            );
                        } else {
                            tracing::info!("Restored {} to {}%", m.display_name(), b);
                        }
                    }
                }
            }
        }

        // Keep tray alive for the app lifetime
        std::mem::forget(tray);

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

    // Reset gamma on all outputs when app exits
    let _ = crate::ddc::SoftwareFilter::reset_all_gamma();

    tracing::info!("Twinkle Linux shutdown complete");
}
