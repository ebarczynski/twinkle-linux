//! Visual preview of the brightness popup for debugging rendering.
//!
//! Usage: `GDK_BACKEND=x11 cargo run --example popup_preview`
//! (renders the popup for ~15 s, then exits — pair it with a screenshot).

use twinkle_linux::core::ConfigManager;
use twinkle_linux::ddc::DDCManager;
use twinkle_linux::ui::brightness_popup::BrightnessPopup;

fn main() {
    gtk4::init().expect("gtk init");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let _guard = rt.enter();

    let parent = gtk4::Window::new();

    let parent_clone = parent.clone();
    gtk4::glib::spawn_future_local(async move {
        let ddc = DDCManager::new().await.expect("ddc manager");
        let config = ConfigManager::new().expect("config manager");
        let popup = BrightnessPopup::new(
            &parent_clone,
            std::sync::Arc::new(ddc),
            std::sync::Arc::new(tokio::sync::Mutex::new(config)),
            None,
        )
        .await;
        popup.popup(); // rebuilds cards and presents the window
        println!("POPUP_PRESENTED");
    });

    // Auto-quit so the example terminates on its own.
    let loop_ = gtk4::glib::MainLoop::new(None, false);
    {
        let loop_ = loop_.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_secs(15), move || {
            loop_.quit();
            glib::ControlFlow::Break
        });
    }

    loop_.run();
}
