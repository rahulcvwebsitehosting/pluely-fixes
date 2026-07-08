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
    // 1. WEBKIT_DISABLE_COMPOSITING_MODE=1  — force WebKit to use CPU-based
    //    compositing, bypassing the GPU EGL path entirely.
    // 2. GDK_BACKEND                         — AppImages bundle GTK/WebKit that
    //    often lacks a Wayland GDK backend, so we force X11 via XWayland.
    //    Source builds get "wayland,x11" so native Wayland is tried first.
    // 3. EGL_PLATFORM=x11                    — tell Mesa to use the X11 EGL
    //    platform so EGL initialisation goes through X11 even when the
    //    display server is Wayland (AppImage-specific).
    // 4. WEBKIT_FORCE_SANDBOX=0              — disable the WebKit process
    //    sandbox inside the AppImage (already confined by the squashfs
    //    mount and bubblewrap/setpriv).
    #[cfg(target_os = "linux")]
    {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            // Always disable accelerated compositing — the root cause of
            // EGL_BAD_PARAMETER with transparent windows on Wayland.
            if std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE").is_err() {
                std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            }

            // AppImages bundle their own GTK+/WebKit which is typically
            // compiled against X11 only, so native Wayland won't work.
            // Source builds have the system GTK which includes Wayland
            // GDK backend.
            if std::env::var("APPIMAGE").is_ok() {
                // AppImage: pure X11 via XWayland
                if std::env::var("GDK_BACKEND").is_err() {
                    std::env::set_var("GDK_BACKEND", "x11");
                }
                if std::env::var("EGL_PLATFORM").is_err() {
                    std::env::set_var("EGL_PLATFORM", "x11");
                }
                if std::env::var("WEBKIT_FORCE_SANDBOX").is_err() {
                    std::env::set_var("WEBKIT_FORCE_SANDBOX", "0");
                }
            } else {
                // Source build: prefer native Wayland, fall back to X11
                if std::env::var("GDK_BACKEND").is_err() {
                    std::env::set_var("GDK_BACKEND", "wayland,x11");
                }
            }
        }
    }

    pluely_lib::run()
}
