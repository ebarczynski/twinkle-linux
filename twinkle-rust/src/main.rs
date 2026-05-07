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
use std::sync::atomic::{AtomicBool, Ordering};
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
    tracing::info!("build_ui: Starting UI construction");

    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Twinkle Linux")
        .default_width(400)
        .default_height(300)
        .build();
    tracing::info!("build_ui: Main window created");

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
    tracing::info!("build_ui: Status label created with 'Initializing...'");

    container.append(&status_label);
    window.set_child(Some(&container));

    // Use an atomic flag to track initialization state
    let initialized = Arc::new(AtomicBool::new(false));
    let initialized_clone = initialized.clone();
    
    // Clone values before moving into async closures
    let ddc_manager_for_init = state.ddc_manager.clone();
    let config_manager_for_init = state.config_manager.clone();
    let ddc_manager_for_ui = state.ddc_manager.clone();
    let config_manager_for_ui = state.config_manager.clone();
    
    // Setup tray icon
    // Clone the Application to move into the async closure
    let app_clone = app.clone();
    tracing::info!("Spawning async initialization task...");
    glib::spawn_future_local(async move {
        tracing::info!("Async initialization task started");
        
        tracing::info!("Creating TrayIcon...");
        let tray_icon = TrayIcon::new(
            app_clone,
            ddc_manager_for_init.clone(),
            config_manager_for_init.clone(),
        )
        .await;
        tracing::info!("TrayIcon created successfully");

        // Initialize DDC manager
        tracing::info!("Initializing DDC manager...");
        match ddc_manager_for_init.initialize().await {
            Ok(true) => {
                tracing::info!("DDC manager initialized successfully");
                tray_icon.update_state().await;
                
                // Mark as initialized
                initialized_clone.store(true, Ordering::SeqCst);
                tracing::info!("Initialization complete");
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
    
    // Setup UI update using glib::timeout_add_local
    let container_clone = container.clone();
    let status_label_clone = status_label.clone();
    let app_clone_for_ui = app.clone();
    let initialized_for_ui = initialized.clone();
    
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if initialized_for_ui.load(Ordering::SeqCst) {
            tracing::info!("build_ui: Processing Initialized message");
            
            // Remove the initializing label
            status_label_clone.set_label("Ready!");
            tracing::info!("build_ui: Status label updated to 'Ready!'");
            
            // Clear the container and add the tray icon widget
            while let Some(child) = container_clone.first_child() {
                container_clone.remove(&child);
            }
            tracing::info!("build_ui: Container cleared");

            // Create and add the tray icon widget
            let tray_icon_widget = gtk4::MenuButton::builder()
                .icon_name("display-brightness-symbolic")
                .tooltip_text("Twinkle Linux - Monitor Brightness Control")
                .build();
            tracing::info!("build_ui: Tray icon button created");

            // Create a label to show the app is ready
            let ready_label = Label::builder()
                .label("Twinkle Linux is running\nCheck the system tray for controls")
                .halign(gtk4::Align::Center)
                .valign(gtk4::Align::Center)
                .margin_top(20)
                .margin_bottom(20)
                .build();
            tracing::info!("build_ui: Ready label created");

            container_clone.append(&tray_icon_widget);
            container_clone.append(&ready_label);
            tracing::info!("build_ui: UI widgets added to container");

            // Initialize the actual tray icon functionality
            let app_for_tray = app_clone_for_ui.clone();
            let ddc_for_tray = ddc_manager_for_ui.clone();
            let config_for_tray = config_manager_for_ui.clone();
            glib::spawn_future_local(async move {
                tracing::info!("build_ui: Creating TrayIcon after UI update");
                let _tray_icon = TrayIcon::new(
                    app_for_tray,
                    ddc_for_tray,
                    config_for_tray,
                ).await;
                tracing::info!("build_ui: TrayIcon created after UI update");
            });
            
            // Stop the timeout
            glib::ControlFlow::Break
        } else {
            // Continue polling
            glib::ControlFlow::Continue
        }
    });

    window.show();
    tracing::info!("build_ui: Window shown");
}

/// Message type for async initialization communication
#[derive(Debug)]
enum InitializationMessage {
    Initialized,
    Error(String),
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
