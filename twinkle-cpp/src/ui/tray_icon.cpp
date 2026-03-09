#include "twinkle/ui/tray_icon.hpp"
#include <glib.h>

namespace twinkle::ui {

TrayIcon::TrayIcon()
    : status_icon_(nullptr, g_object_unref) {}

TrayIcon::~TrayIcon() {
    hide();
}

bool TrayIcon::initialize() {
    // Create status icon
    status_icon_.reset(gtk_status_icon_new("twinkle-linux"));

    if (!status_icon_) {
        return false;
    }

    // Set icon (using a simple icon for now)
    GdkPixbuf* pixbuf = gdk_pixbuf_new(GDK_COLORSPACE_RGB, TRUE, 8, 32, 32);
    if (pixbuf) {
        gtk_status_icon_set_from_pixbuf(status_icon_.get(), pixbuf);
        g_object_unref(pixbuf);
    }

    // Set tooltip
    gtk_status_icon_set_tooltip_text(status_icon_.get(), "Twinkle Linux - Brightness Control");

    // Connect signals
    g_signal_connect(status_icon_.get(), "activate",
                   G_CALLBACK(on_tray_click), this);
    g_signal_connect(status_icon_.get(), "popup-menu",
                   G_CALLBACK(on_tray_click), this);

    return true;
}

void TrayIcon::show() {
    if (status_icon_) {
        gtk_status_icon_set_visible(status_icon_.get(), TRUE);
    }
}

void TrayIcon::hide() {
    if (status_icon_) {
        gtk_status_icon_set_visible(status_icon_.get(), FALSE);
    }
}

void TrayIcon::update_icon(bool monitors_available) {
    // Update icon based on monitor state
    // For now, just show/hide
    if (monitors_available) {
        show();
    } else {
        hide();
    }
}

GtkWidget* TrayIcon::create_menu() {
    GtkWidget* menu = gtk_menu_new();

    // Brightness Control
    GtkWidget* brightness_item = gtk_menu_item_new_with_label("Brightness Control");
    g_signal_connect(brightness_item, "activate",
                   G_CALLBACK(on_menu_item_click), this);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), brightness_item);

    // Separator
    GtkWidget* separator = gtk_separator_menu_item_new();
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), separator);

    // Settings
    GtkWidget* settings_item = gtk_menu_item_new_with_label("Settings");
    g_signal_connect(settings_item, "activate",
                   G_CALLBACK(on_menu_item_click), this);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), settings_item);

    // Quit
    GtkWidget* quit_item = gtk_image_menu_item_new_from_stock(GTK_STOCK_QUIT, nullptr);
    g_signal_connect(quit_item, "activate",
                   G_CALLBACK(on_menu_item_click), this);
    gtk_menu_shell_append(GTK_MENU_SHELL(menu), quit_item);

    gtk_widget_show_all(menu);

    return menu;
}

void TrayIcon::on_menu_item_click(GtkWidget* widget, gpointer data) {
    TrayIcon* self = static_cast<TrayIcon*>(data);
    const gchar* label = gtk_menu_item_get_label(GTK_MENU_ITEM(widget));

    if (label) {
        if (strcmp(label, "Brightness Control") == 0) {
            if (self->brightness_callback_) {
                self->brightness_callback_();
            }
        } else if (strcmp(label, "Settings") == 0) {
            if (self->settings_callback_) {
                self->settings_callback_();
            }
        }
    } else {
        // Quit item (no label)
        if (self->quit_callback_) {
            self->quit_callback_();
        }
    }
}

void TrayIcon::on_tray_click(GtkStatusIcon* status_icon, gpointer user_data) {
    TrayIcon* self = static_cast<TrayIcon*>(user_data);

    // Show menu on right-click, brightness popup on left-click
    guint button;
    guint activate_time;
    gdk_event_get_button(gdk_event_get(), &button, nullptr);
    activate_time = gtk_get_current_event_time();

    if (button == 3) {
        // Right-click - show menu
        GtkWidget* menu = self->create_menu();
        gtk_menu_popup(GTK_MENU(menu), nullptr, nullptr, nullptr, nullptr,
                      button, activate_time);
    } else {
        // Left-click - show brightness popup
        if (self->brightness_callback_) {
            self->brightness_callback_();
        }
    }
}

} // namespace twinkle::ui
