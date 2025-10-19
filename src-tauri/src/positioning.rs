#![cfg_attr(target_os = "macos", allow(unexpected_cfgs))]
use crate::errors::EmojiError;
use std::sync::{Arc, Mutex};
#[cfg(not(target_os = "macos"))]
use tauri::{PhysicalPosition, Position};

#[cfg(target_os = "macos")]
use crate::constants::APP_BUNDLE_IDENTIFIER;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSString};
#[cfg(target_os = "macos")]
use dispatch::Queue;
#[cfg(target_os = "macos")]
use libc::pthread_main_np;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

// PositioningError has been replaced with EmojiError for consistency

// Global storage for the previously active application
lazy_static::lazy_static! {
    static ref PREVIOUS_APP: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

/// Position panel at cursor using the window directly
/// This function works by taking a Tauri WebviewWindow and positioning it smartly
/// It avoids the dock and menu bar by using the visible screen area
/// macOS positioning
/// I use Cocoa and Objective-C for this because, with the previous Tauri implementation, the panel would sometimes initially appear at its previous location before quickly moving to the new position.
#[cfg(target_os = "macos")]
pub fn position_window_at_cursor(window: &tauri::WebviewWindow) -> Result<(), EmojiError> {
    if let Ok(native_window) = window.ns_window() {
        use cocoa::foundation::NSPoint;
        use objc::{msg_send, sel, sel_impl};

        let ns_window = native_window as id;

        // Get cursor position and window/screen info (unsafe Cocoa calls)
        let (raw_mouse_location, window_width, window_height, screen_visible_frame) = unsafe {
            let raw_mouse_location: NSPoint =
                cocoa::appkit::NSEvent::mouseLocation(std::ptr::null_mut());

            let window_frame: cocoa::foundation::NSRect = msg_send![ns_window, frame];
            let window_width = window_frame.size.width;
            let window_height = window_frame.size.height;

            let main_screen = cocoa::appkit::NSScreen::mainScreen(std::ptr::null_mut());
            let screen_visible_frame = cocoa::appkit::NSScreen::visibleFrame(main_screen);

            (
                raw_mouse_location,
                window_width,
                window_height,
                screen_visible_frame,
            )
        };

        // Calculate positioning (safe calculations)
        let desired_x = raw_mouse_location.x;
        let desired_y = raw_mouse_location.y - window_height;

        // Check bounds and adjust if necessary (keep window on visible screen)
        let final_x = desired_x
            .max(screen_visible_frame.origin.x) // Don't go left of visible area
            .min(screen_visible_frame.origin.x + screen_visible_frame.size.width - window_width); // Don't go right of visible area

        let final_y = desired_y
            .max(screen_visible_frame.origin.y) // Don't go below visible area
            .min(screen_visible_frame.origin.y + screen_visible_frame.size.height - window_height); // Don't go above visible area

        let cocoa_point = NSPoint {
            x: final_x,
            y: final_y,
        };

        // Apply positioning (unsafe Cocoa call)
        unsafe {
            let _: () = msg_send![ns_window, setFrameOrigin: cocoa_point];
        }

        Ok(())
    } else {
        log::error!("Failed to get native window for macOS positioning");
        Err(EmojiError::WindowHandle)
    }
}

// Function to store the currently active application
#[cfg(target_os = "macos")]
pub fn store_previous_app() {
    log::debug!("Storing previous app...");

    #[cfg(target_os = "macos")]
    unsafe {
        let is_main = pthread_main_np() != 0;
        let work = || {
            let _pool = NSAutoreleasePool::new(nil);
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let frontmost: id = msg_send![workspace, frontmostApplication];
            if frontmost != nil {
                let bundle_id_ns: id = msg_send![frontmost, bundleIdentifier];
                if bundle_id_ns != nil {
                    let c_str = NSString::UTF8String(bundle_id_ns);
                    if !c_str.is_null() {
                        let bundle_id = std::ffi::CStr::from_ptr(c_str)
                            .to_string_lossy()
                            .into_owned();
                        if !bundle_id.is_empty() && bundle_id != APP_BUNDLE_IDENTIFIER {
                            if let Ok(mut previous_app) = PREVIOUS_APP.lock() {
                                *previous_app = Some(bundle_id.clone());
                                log::debug!("Stored previous app: {}", bundle_id);
                            }
                        }
                    }
                }
            }
        };
        if is_main {
            work();
        } else {
            Queue::main().exec_sync(work);
        }
    }
}

// Function to restore focus to the previously active application
#[cfg(target_os = "macos")]
pub fn restore_previous_app() {
    log::debug!("Restoring previous app...");

    if let Ok(previous_app) = PREVIOUS_APP.lock() {
        if let Some(bundle_id) = previous_app.as_ref() {
            log::debug!("Restoring focus to: {}", bundle_id);
            // Use native Cocoa APIs to activate the app on the main thread
            #[cfg(target_os = "macos")]
            {
                let bundle_id_owned = bundle_id.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(
                        crate::constants::FOCUS_RESTORATION_DELAY_MS,
                    ));
                    Queue::main().exec_async(move || unsafe {
                        let _pool = NSAutoreleasePool::new(nil);
                        let ns_str: id = NSString::alloc(nil).init_str(&bundle_id_owned);
                        // Use the correct API: runningApplicationsWithBundleIdentifier:
                        let apps: id = msg_send![class!(NSRunningApplication), runningApplicationsWithBundleIdentifier: ns_str];
                        if apps != nil {
                            let count = cocoa::foundation::NSArray::count(apps);
                            if count > 0 {
                                let app = cocoa::foundation::NSArray::objectAtIndex(apps, 0);
                                // NSApplicationActivateIgnoringOtherApps = 1
                                let _: bool = msg_send![app, activateWithOptions: 1u64];
                            } else {
                                log::warn!("No running app found with bundle id: {}", bundle_id_owned);
                            }
                        } else {
                            log::warn!("No running app array for bundle id: {}", bundle_id_owned);
                        }
                    });
                });
            }
        } else {
            log::debug!("No previous app stored");
        }
    }
}

// Non-macOS positioning
// Not tested, partially implemented, so commented for the great future

// // Simple rect structure for non-macOS platforms
// #[cfg(not(target_os = "macos"))]
// #[derive(Debug, Clone, Copy)]
// struct SimpleRect {
//     x: f64,
//     y: f64,
//     width: f64,
//     height: f64,
// }

// // Trait to unify rect access across platforms
// #[cfg(not(target_os = "macos"))]
// impl RectAccess for SimpleRect {
//     fn left(&self) -> f64 {
//         self.x
//     }
//     fn top(&self) -> f64 {
//         self.y
//     }
//     fn width(&self) -> f64 {
//         self.width
//     }
//     fn height(&self) -> f64 {
//         self.height
//     }
// }

// #[cfg(not(target_os = "macos"))]
// fn get_visible_screen_area() -> Result<(SimpleRect, f64), PositioningError> {
//     // Fallback to monitor API for non-macOS
//     let monitor = get_monitor_with_cursor().ok_or(PositioningError::MonitorNotFound)?;
//     let monitor_scale_factor = monitor.scale_factor();
//     let monitor_size = monitor.size().to_logical::<f64>(monitor_scale_factor);
//     let monitor_position = monitor.position().to_logical::<f64>(monitor_scale_factor);

//     let rect = SimpleRect {
//         x: monitor_position.x,
//         y: monitor_position.y,
//         width: monitor_size.width,
//         height: monitor_size.height,
//     };
//     Ok((rect, monitor_scale_factor))
// }

// #[cfg(not(target_os = "macos"))]
// pub fn position_window_at_cursor(window: &tauri::WebviewWindow) -> Result<(), PositioningError> {
//     {
//         // Fallback to Tauri positioning (for non-macOS)
//         // Get cursor position in screen coordinates

//         // TODO: Implement this
//         Err(PositioningError::MonitorNotFound);
//         // let cursor_pos = get_cursor_position()?;

//         // Get panel size using Tauri API
//         let panel_size = window
//             .outer_size()
//             .map_err(|_| PositioningError::WindowHandleError)?;

//         // Get visible screen area (excluding dock and menu bar) for the screen with cursor
//         let (visible_area, scale_factor) = get_visible_screen_area()?;

//         // Convert panel size to logical pixels to match cursor coordinates
//         let panel_logical_size = PhysicalPosition {
//             x: panel_size.width as f64 / scale_factor,
//             y: panel_size.height as f64 / scale_factor,
//         };

//         // Calculate visible area bounds in logical coordinates (to match cursor)
//         let visible_left = visible_area.left();
//         let visible_top = visible_area.top();
//         let visible_right = visible_left + visible_area.width();
//         let visible_bottom = visible_top + visible_area.height();

//         // Calculate if panel fits in each direction from cursor (using logical coordinates)
//         let fits_right = cursor_pos.x + panel_logical_size.x <= visible_right;
//         let fits_below = cursor_pos.y + panel_logical_size.y <= visible_bottom;

//         // Determine panel position based on available space (using logical coordinates)
//         let panel_x = if fits_right {
//             cursor_pos.x // Top-left or bottom-left at cursor
//         } else {
//             cursor_pos.x - panel_logical_size.x // Top-right or bottom-right at cursor
//         };

//         let panel_y = if fits_below {
//             cursor_pos.y // Top-left or top-right at cursor
//         } else {
//             cursor_pos.y - panel_logical_size.y // Bottom-left or bottom-right at cursor
//         };

//         // Ensure the panel stays within visible area bounds (safety clamp, using logical coordinates)
//         let final_x = panel_x
//             .max(visible_left)
//             .min(visible_right - panel_logical_size.x);
//         let final_y = panel_y
//             .max(visible_top)
//             .min(visible_bottom - panel_logical_size.y);

//         // Convert back to physical coordinates for Tauri API
//         let physical_x = (final_x * scale_factor) as i32;
//         let physical_y = (final_y * scale_factor) as i32;

//         // Set panel position using Tauri API
//         let position = Position::Physical(PhysicalPosition {
//             x: physical_x,
//             y: physical_y,
//         });

//         window
//             .set_position(position)
//             .map_err(|_| PositioningError::WindowHandleError)?;

//         Ok(())
//     }
// }

// #[cfg(not(target_os = "macos"))]
// pub fn store_previous_app() {
//     // No-op for non-macOS platforms
// }

// #[cfg(not(target_os = "macos"))]
// pub fn restore_previous_app() {
//     // No-op for non-macOS platforms
// }

// #[cfg(not(target_os = "macos"))]
// use monitor::get_monitor_with_cursor;
