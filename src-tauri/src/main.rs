// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Custom panic hook — in release mode with windows_subsystem = "windows",
    // panics silently exit the process with no terminal output. This hook
    // ensures the panic message is flushed to stderr (visible when launched
    // from a terminal) before the process terminates.
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            format!("{:?}", info.payload())
        };
        let location = info
            .location()
            .map(|l| format!(" at {}:{}", l.file(), l.line()))
            .unwrap_or_default();
        eprintln!("[pluely] PANIC{location}: {msg}");
        use std::io::Write;
        let _ = std::io::stderr().flush();
    }));

    // Wayland compatibility: transparent Tauri windows fail with EGL_BAD_PARAMETER
    // on Wayland compositors (KDE, GNOME) because WebKitGTK's EGL context
    // initialisation rejects the pixel format when transparency is enabled.
    //
    // WEBKIT_DISABLE_COMPOSITING_MODE=1 forces WebKit to use a non-accelerated
    // compositing path that works correctly under Wayland, at the cost of
    // slightly higher CPU usage for the overlay window.
    //
    // GDK_BACKEND=wayland,x11 tells GTK to prefer Wayland but fall back to X11
    // if the Wayland compositor does not support the required EGL extensions.
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
                std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            }
            if std::env::var("GDK_BACKEND").is_err() {
                std::env::set_var("GDK_BACKEND", "wayland,x11");
            }
        }
    }

    pluely_lib::run()
}
