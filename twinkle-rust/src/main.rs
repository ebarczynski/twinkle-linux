//! Twinkle Linux - GUI application for controlling external monitor brightness via DDC/CI on Linux.

mod core;
mod ddc;
mod ui;
mod utils;

use crate::core::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::{setup_tray_actions, TrayIcon};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Label, Orientation};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application state.
struct AppState {
    /// DDC manager
    ddc_manager: Arc<DDCManager>,
    /// Config manager
    config_manager: Arc<Mutex<ConfigManager>>,
}

/// Build the application UI.
fn build_ui(app: &Application, state: AppState) {
    // Create main window
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

    // Setup tray icon
    // Clone the Application to move into the async closure
    let app_clone = app.clone();
    tracing::info!("Spawning async initialization task...");
    glib::spawn_future_local(async move {
        tracing::info!("Async initialization task started");
        
        tracing::info!("Creating TrayIcon...");
        let tray_icon = TrayIcon::new(
            app_clone,
            state.ddc_manager.clone(),
            state.config_manager.clone(),
        )
        .await;
        tracing::info!("TrayIcon created successfully");

        // Initialize DDC manager
        tracing::info!("Initializing DDC manager...");
        match state.ddc_manager.initialize().await {
            Ok(true) => {
                tracing::info!("DDC manager initialized successfully");
                tray_icon.update_state().await;
            }
            Ok(false) => {
                tracing::warn!("DDC manager initialization failed");
            }
            Err(e) => {
                tracing::error!("DDC manager initialization error: {}", e);
            }
        }
        tracing::info!("Async initialization task completed");
    });

    window.show();
}

/// Main entry point.
#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting Twinkle Linux v{}", utils::version());

    // Create application
    let app = Application::builder()
        .application_id("com.github.twinkle-twinkle.TwinkleLinux")
        .build();

    // Setup application actions
    setup_tray_actions(&app);

    // Connect activate signal
    app.connect_activate(|app| {
        tracing::info!("App activate callback started");
        
        // Create config manager
        tracing::info!("Creating ConfigManager...");
        let config_manager = Arc::new(Mutex::new(
            ConfigManager::new().expect("Failed to create config manager"),
        ));
        tracing::info!("ConfigManager created successfully");

        // Create DDC manager
        tracing::info!("Creating DDCManager...");
        let ddc_manager = Arc::new(
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(DDCManager::new())
            })
            .expect("Failed to create DDC manager"),
        );
        tracing::info!("DDCManager created successfully");

        let state = AppState {
            ddc_manager,
            config_manager,
        };

        tracing::info!("Calling build_ui...");
        build_ui(app, state);
        tracing::info!("build_ui completed");
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);

    tracing::info!("Twinkle Linux shutdown complete");
}
