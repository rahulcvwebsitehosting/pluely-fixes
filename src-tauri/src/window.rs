#[cfg(target_os = "macos")]
use tauri::LogicalPosition;
#[cfg(target_os = "linux")]
use tauri::window::WindowExtUnix;
use tauri::{App, AppHandle, Manager, Runtime, WebviewWindow, WebviewWindowBuilder};

// The offset from the top of the screen to the window
const TOP_OFFSET: i32 = 54;

/// Sets up the main window with custom positioning
pub fn setup_main_window(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Try different possible window labels
    let window = app
        .get_webview_window("main")
        .or_else(|| app.get_webview_window("pluely"))
        .or_else(|| {
            // Get the first window if specific labels don't work
            app.webview_windows().values().next().cloned()
        })
        .ok_or("No window found")?;

    position_window_top_center(&window, TOP_OFFSET)?;

    // Keep the main overlay non-focusable on Windows so showing/toggling it
    // does not steal OS focus. This keeps global shortcuts (e.g. Ctrl+Arrow
    // to move the window) reliable during live meetings/interviews.
    ensure_main_window_non_focusable(&window);

    // On Linux, configure window hints so screen-capture and screen-sharing
    // tools (Zoom, Google Meet/Chrome, GNOME Screencast, OBS, etc.) exclude
    // the overlay from the capture source — this is the core stealth feature.
    #[cfg(target_os = "linux")]
    hide_from_screenshare(&window);

    Ok(())
}

/// Keeps the overlay from becoming the active OS window on platforms that support it.
#[cfg(target_os = "windows")]
pub fn ensure_main_window_non_focusable<R: Runtime>(window: &WebviewWindow<R>) {
    if let Err(e) = window.set_focusable(false) {
        eprintln!("Failed to set main window non-focusable: {}", e);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_main_window_non_focusable<R: Runtime>(_window: &WebviewWindow<R>) {}

/// On Windows: re-asserts the non-focusable (`WS_EX_NOACTIVATE`) state after any
/// user interaction (click, keypress, drag) so the overlay never steals focus
/// from fullscreen apps (browsers, proctoring, meetings).
#[tauri::command]
pub fn reset_focusable<R: Runtime>(window: tauri::WebviewWindow<R>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    ensure_main_window_non_focusable(&window);
    Ok(())
}

/// Shows the main overlay without requesting OS focus.
///
/// On Windows this uses `ShowWindow(SW_SHOWNOACTIVATE)` instead of Tauri's
/// `window.show()` (which calls `SW_SHOW` and **always** activates the
/// window regardless of `WS_EX_NOACTIVATE`).  Combined with
/// `set_focusable(false)` this prevents focus steal on show, mouse click,
/// and keyboard input.
pub fn show_main_window_without_focus<R: Runtime>(
    window: &WebviewWindow<R>,
) -> Result<(), String> {
    ensure_main_window_non_focusable(window);

    #[cfg(target_os = "windows")]
    show_window_no_activate(window)?;
    #[cfg(not(target_os = "windows"))]
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;

    ensure_main_window_non_focusable(window);

    Ok(())
}

/// Windows-only: shows the window via `ShowWindow(SW_SHOWNOACTIVATE)` so the
/// overlay never steals focus from the currently active application.
///
/// Tauri's built-in `window.show()` → `ShowWindow(SW_SHOW)` always activates
/// the window regardless of the `WS_EX_NOACTIVATE` extended style, causing
/// the previously active application to receive a `WM_ACTIVATE` / blur event.
/// This is the root cause of ##131 and the stealth-mode focus-leak bug.
#[cfg(target_os = "windows")]
fn show_window_no_activate<R: Runtime>(window: &WebviewWindow<R>) -> Result<(), String> {
    use std::ffi::c_void;
    use tauri::raw_window_handle::HasRawWindowHandle;

    match window.raw_window_handle() {
        Ok(raw) => {
            if let tauri::raw_window_handle::RawWindowHandle::Win32(win) = raw {
                let hwnd = win.hwnd.as_ptr() as *mut c_void;

                extern "system" {
                    fn ShowWindow(hWnd: *mut c_void, nCmdShow: i32) -> i32;
                }
                // SW_SHOWNOACTIVATE = 8 — shows the window but does not
                // activate it; the previously foreground window stays active.
                const SW_SHOWNA: i32 = 8;
                unsafe { ShowWindow(hwnd, SW_SHOWNA); }

                return Ok(());
            }
        }
        Err(e) => {
            eprintln!(
                "Failed to get raw window handle, falling back to Tauri show(): {:?}",
                e
            );
        }
    }

    // Fallback: Tauri's normal show (will briefly activate, but better than nothing)
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))
}

/// Positions a window at the top center of the screen with a specified Y offset
pub fn position_window_top_center(
    window: &WebviewWindow,
    y_offset: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the primary monitor
    if let Some(monitor) = window.primary_monitor()? {
        let monitor_size = monitor.size();
        let window_size = window.outer_size()?;

        // Calculate center X position
        let center_x = (monitor_size.width as i32 - window_size.width as i32) / 2;

        // Set the window position
        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: center_x,
            y: y_offset,
        }))?;
    }

    Ok(())
}

/// Future function for centering window completely (both X and Y)
#[allow(dead_code)]
pub fn center_window_completely(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(monitor) = window.primary_monitor()? {
        let monitor_size = monitor.size();
        let window_size = window.outer_size()?;

        let center_x = (monitor_size.width as i32 - window_size.width as i32) / 2;
        let center_y = (monitor_size.height as i32 - window_size.height as i32) / 2;

        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: center_x,
            y: center_y,
        }))?;
    }

    Ok(())
}

#[tauri::command]
pub fn set_window_size(
    window: tauri::WebviewWindow,
    width: u32,
    height: u32,
) -> Result<(), String> {
    use tauri::{LogicalSize, Size};

    let new_size = LogicalSize::new(width as f64, height as f64);
    window
        .set_size(Size::Logical(new_size))
        .map_err(|e| format!("Failed to resize window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn set_window_height(window: tauri::WebviewWindow, height: u32) -> Result<(), String> {
    use tauri::{LogicalSize, Size};

    let new_size = LogicalSize::new(800.0, height as f64);
    window
        .set_size(Size::Logical(new_size))
        .map_err(|e| format!("Failed to resize window: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn open_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    show_dashboard_window(&app)
}

#[tauri::command]
pub fn toggle_dashboard(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(dashboard_window) = app.get_webview_window("dashboard") {
        match dashboard_window.is_visible() {
            Ok(true) => {
                // Window is visible, hide it
                dashboard_window
                    .hide()
                    .map_err(|e| format!("Failed to hide dashboard window: {}", e))?;
            }
            Ok(false) => {
                // Window is hidden, show and focus it
                dashboard_window
                    .show()
                    .map_err(|e| format!("Failed to show dashboard window: {}", e))?;
                dashboard_window
                    .set_focus()
                    .map_err(|e| format!("Failed to focus dashboard window: {}", e))?;
            }
            Err(e) => {
                return Err(format!("Failed to check dashboard visibility: {}", e));
            }
        }
    } else {
        // Window doesn't exist, create and show it
        show_dashboard_window(&app)?;
    }

    Ok(())
}

#[tauri::command]
pub fn move_window(app: tauri::AppHandle, direction: String, step: i32) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let current_pos = window
            .outer_position()
            .map_err(|e| format!("Failed to get window position: {}", e))?;

        let (new_x, new_y) = match direction.as_str() {
            "up" => (current_pos.x, current_pos.y - step),
            "down" => (current_pos.x, current_pos.y + step),
            "left" => (current_pos.x - step, current_pos.y),
            "right" => (current_pos.x + step, current_pos.y),
            _ => return Err(format!("Invalid direction: {}", direction)),
        };

        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: new_x,
                y: new_y,
            }))
            .map_err(|e| format!("Failed to set window position: {}", e))?;
    } else {
        return Err("Main window not found".to_string());
    }

    Ok(())
}

pub fn create_dashboard_window<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<WebviewWindow<R>, tauri::Error> {
    let base_builder =
        WebviewWindowBuilder::new(app, "dashboard", tauri::WebviewUrl::App("/chats".into()));

    #[cfg(target_os = "macos")]
    let base_builder = base_builder
        .title("Pluely - Dashboard")
        .center()
        .decorations(true)
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .hidden_title(true)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .content_protected(true)
        .visible(true)
        .traffic_light_position(LogicalPosition::new(14.0, 18.0));

    #[cfg(not(target_os = "macos"))]
    let base_builder = base_builder
        .title("Pluely - Dashboard")
        .center()
        .decorations(true)
        .inner_size(800.0, 600.0)
        .min_inner_size(800.0, 600.0)
        .content_protected(true)
        .visible(false);

    let window = base_builder.build()?;

    // Set up close event handler - hide window instead of destroying it
    setup_dashboard_close_handler(&window);

    Ok(window)
}

/// Sets up the close event handler for the dashboard window
fn setup_dashboard_close_handler<R: Runtime>(window: &WebviewWindow<R>) {
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            // Prevent the window from being destroyed
            api.prevent_close();
            // Hide the window instead
            if let Err(e) = window_clone.hide() {
                eprintln!("Failed to hide dashboard window on close: {}", e);
            }
        }
    });
}

/// Shows the dashboard window and brings it to focus
pub fn show_dashboard_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(dashboard_window) = app.get_webview_window("dashboard") {
        // Window exists, show and focus it
        dashboard_window
            .show()
            .map_err(|e| format!("Failed to show dashboard window: {}", e))?;
        dashboard_window
            .set_focus()
            .map_err(|e| format!("Failed to focus dashboard window: {}", e))?;
    } else {
        // Window doesn't exist, create it and then show it
        let window = create_dashboard_window(app)
            .map_err(|e| format!("Failed to create dashboard window: {}", e))?;
        window
            .show()
            .map_err(|e| format!("Failed to show new dashboard window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus new dashboard window: {}", e))?;
    }
    Ok(())
}

/// On macOS: temporarily activates the panel so WKWebView will show the native
/// file-picker dialog.  Non-activating panels cannot open modal sheets/dialogs
/// (the file picker simply fails silently), so we briefly make the panel key.
#[tauri::command]
pub fn activate_window_for_file_picker<R: Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        window.set_focusable(true).map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Restores the non-focusable state after the file picker dialog has closed.
/// Must be paired with `activate_window_for_file_picker`.
#[tauri::command]
pub fn deactivate_window_after_file_picker<R: Runtime>(
    window: tauri::WebviewWindow<R>,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        window.set_focusable(false).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Replaces the app icon at runtime from a PNG file path.
///
/// On macOS this sets `NSApplication.sharedApplication.applicationIconImage`.
/// On Windows it sets the window class icon via `WM_SETICON`.
/// On Linux this is a no-op (icons are set at the `.desktop` level).
#[tauri::command]
pub fn set_app_icon_path(app: tauri::AppHandle, path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use tauri_nspanel::objc::{class, msg_send};
        use tauri_nspanel::objc::runtime::Object;

        let shared_app: *mut Object = unsafe { msg_send![class!(NSApplication), sharedApplication] };

        if path.is_empty() {
            let _: () = unsafe { msg_send![shared_app, setApplicationIconImage: std::ptr::null_mut::<Object>()] };
            return Ok(());
        }

        let ns_str: *mut Object = unsafe {
            msg_send![class!(NSString), stringWithUTF8String: path.as_ptr() as *const i8]
        };
        let data: *mut Object = unsafe { msg_send![class!(NSData), dataWithContentsOfFile: ns_str] };
        if data.is_null() {
            return Err("Failed to read icon file".into());
        }
        let img: *mut Object = unsafe { msg_send![class!(NSImage), alloc] };
        let img: *mut Object = unsafe { msg_send![img, initWithData: data] };
        let _: () = unsafe { msg_send![shared_app, setApplicationIconImage: img] };
    }

    #[cfg(target_os = "windows")]
    {
        use std::ffi::CString;
        use std::ptr;
        use tauri::raw_window_handle::HasRawWindowHandle;

        let main = app.get_webview_window("main");
        let hwnd = match main.and_then(|w| w.raw_window_handle().ok()) {
            Some(tauri::raw_window_handle::RawWindowHandle::Win32(win)) => win.hwnd.as_ptr(),
            _ => return Err("No main window found".into()),
        };

        extern "system" {
            fn LoadImageA(
                hInst: *mut std::ffi::c_void,
                name: *const u8,
                typ: u32,
                cx: i32,
                cy: i32,
                fuLoad: u32,
            ) -> *mut std::ffi::c_void;
            fn SendMessageA(
                hWnd: *mut std::ffi::c_void,
                msg: u32,
                wParam: usize,
                lParam: isize,
            ) -> isize;
            fn DestroyIcon(hIcon: *mut std::ffi::c_void) -> i32;
        }

        const IMAGE_ICON: u32 = 1;
        const LR_LOADFROMFILE: u32 = 0x00000010;
        const LR_DEFAULTSIZE: u32 = 0x00000040;
        const WM_SETICON: u32 = 0x0080;
        const ICON_SMALL: usize = 0;
        const ICON_BIG: usize = 1;

        if path.is_empty() {
            return Ok(());
        }

        let cpath = CString::new(path.as_str()).map_err(|e| e.to_string())?;
        unsafe {
            let hIcon = LoadImageA(
                ptr::null_mut(),
                cpath.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            );
            if hIcon.is_null() {
                return Err("Failed to load icon file".into());
            }
            SendMessageA(hwnd, WM_SETICON, ICON_SMALL, hIcon as isize);
            SendMessageA(hwnd, WM_SETICON, ICON_BIG, hIcon as isize);
            DestroyIcon(hIcon);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = (app, path);
    }

    Ok(())
}

/// On Linux, configures the overlay window so that screen-capture and
/// screen-sharing tools (Zoom, Google Meet, GNOME Screencast, OBS, etc.)
/// exclude it from the captured output.
///
/// ## Why this works
/// The `_NET_WM_WINDOW_TYPE_DOCK` hint tells the compositor the window is a
/// panel/dock — most desktop portals and screen-capture APIs filter out dock
/// windows by default.  Combined with the skip-taskbar/skip-pager hints the
/// window becomes invisible to PipeWire-based capture (Wayland) and
/// X-Shm/Xfixes-based capture (X11).
#[cfg(target_os = "linux")]
pub fn hide_from_screenshare<R: Runtime>(window: &WebviewWindow<R>) {
    use gtk::gdk;
    use gtk::prelude::*;

    let gtk_win = window.gtk_window();
    gtk_win.set_type_hint(gdk::WindowTypeHint::Dock);
    gtk_win.set_skip_taskbar_hint(true);
    gtk_win.set_skip_pager_hint(true);
}
