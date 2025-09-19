#![cfg_attr(target_os = "macos", allow(unexpected_cfgs))]
use std::sync::{Arc, Mutex};
use tauri::{PhysicalPosition, Position};

#[cfg(target_os = "macos")]
use crate::constants::APP_BUNDLE_IDENTIFIER;
#[cfg(target_os = "macos")]
use cocoa::appkit::NSScreen;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSString};
#[cfg(target_os = "macos")]
use dispatch::Queue;
#[cfg(target_os = "macos")]
use libc::pthread_main_np;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(not(target_os = "macos"))]
use monitor::get_monitor_with_cursor;

// Global storage for the previously active application
lazy_static::lazy_static! {
    static ref PREVIOUS_APP: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

// Simple rect structure for non-macOS platforms
#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Copy)]
struct SimpleRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// Trait to unify rect access across platforms
trait RectAccess {
    fn left(&self) -> f64;
    fn top(&self) -> f64;
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}

#[cfg(target_os = "macos")]
impl RectAccess for NSRect {
    fn left(&self) -> f64 {
        self.origin.x
    }
    fn top(&self) -> f64 {
        self.origin.y
    }
    fn width(&self) -> f64 {
        self.size.width
    }
    fn height(&self) -> f64 {
        self.size.height
    }
}

#[cfg(not(target_os = "macos"))]
impl RectAccess for SimpleRect {
    fn left(&self) -> f64 {
        self.x
    }
    fn top(&self) -> f64 {
        self.y
    }
    fn width(&self) -> f64 {
        self.width
    }
    fn height(&self) -> f64 {
        self.height
    }
}

#[derive(Debug)]
pub enum PositioningError {
    MonitorNotFound,
    WindowHandleError,
}

impl std::fmt::Display for PositioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PositioningError::MonitorNotFound => write!(f, "Monitor with cursor not found"),
            PositioningError::WindowHandleError => write!(f, "Failed to get window handle"),
        }
    }
}

impl std::error::Error for PositioningError {}

/// Get the visible screen area (excluding dock and menu bar) for the screen containing the cursor
#[cfg(target_os = "macos")]
fn get_visible_screen_area() -> Result<(NSRect, f64), PositioningError> {
    unsafe {
        // Get cursor position first
        let cursor_pos = get_cursor_position()?;

        // Get all screens
        let screens = NSScreen::screens(std::ptr::null_mut());
        let screen_count = cocoa::foundation::NSArray::count(screens);

        // Find the screen containing the cursor
        for i in 0..screen_count {
            let screen = cocoa::foundation::NSArray::objectAtIndex(screens, i);
            let screen_frame = NSScreen::frame(screen);
            let screen_visible_frame = NSScreen::visibleFrame(screen);
            let scale_factor = NSScreen::backingScaleFactor(screen);

            // Convert screen bounds to top-left origin coordinates (matching cursor)
            let screen_top = screen_frame.size.height - screen_frame.origin.y;
            let screen_bottom = screen_top - screen_frame.size.height;
            let screen_left = screen_frame.origin.x;
            let screen_right = screen_left + screen_frame.size.width;

            // Check if cursor is within this screen
            if cursor_pos.x >= screen_left
                && cursor_pos.x <= screen_right
                && cursor_pos.y >= screen_bottom
                && cursor_pos.y <= screen_top
            {
                // Convert visible frame to top-left origin coordinates
                let visible_frame = NSRect {
                    origin: NSPoint {
                        x: screen_visible_frame.origin.x,
                        y: screen_frame.size.height
                            - screen_visible_frame.origin.y
                            - screen_visible_frame.size.height,
                    },
                    size: screen_visible_frame.size,
                };

                return Ok((visible_frame, scale_factor));
            }
        }

        Err(PositioningError::MonitorNotFound)
    }
}

#[cfg(not(target_os = "macos"))]
fn get_visible_screen_area() -> Result<(SimpleRect, f64), PositioningError> {
    // Fallback to monitor API for non-macOS
    let monitor = get_monitor_with_cursor().ok_or(PositioningError::MonitorNotFound)?;
    let monitor_scale_factor = monitor.scale_factor();
    let monitor_size = monitor.size().to_logical::<f64>(monitor_scale_factor);
    let monitor_position = monitor.position().to_logical::<f64>(monitor_scale_factor);

    let rect = SimpleRect {
        x: monitor_position.x,
        y: monitor_position.y,
        width: monitor_size.width,
        height: monitor_size.height,
    };
    Ok((rect, monitor_scale_factor))
}

/// Position panel at cursor using the window directly
/// This function works by taking a Tauri WebviewWindow and positioning it smartly
/// It avoids the dock and menu bar by using the visible screen area
pub fn position_window_at_cursor(window: &tauri::WebviewWindow) -> Result<(), PositioningError> {
    // Get cursor position in screen coordinates
    let cursor_pos = get_cursor_position()?;

    // Get panel size using Tauri API
    let panel_size = window
        .outer_size()
        .map_err(|_| PositioningError::WindowHandleError)?;

    // Get visible screen area (excluding dock and menu bar) for the screen with cursor
    let (visible_area, scale_factor) = get_visible_screen_area()?;

    // Convert panel size to logical pixels to match cursor coordinates
    let panel_logical_size = PhysicalPosition {
        x: panel_size.width as f64 / scale_factor,
        y: panel_size.height as f64 / scale_factor,
    };

    // Calculate visible area bounds in logical coordinates (to match cursor)
    let visible_left = visible_area.left();
    let visible_top = visible_area.top();
    let visible_right = visible_left + visible_area.width();
    let visible_bottom = visible_top + visible_area.height();

    // Calculate if panel fits in each direction from cursor (using logical coordinates)
    let fits_right = cursor_pos.x + panel_logical_size.x <= visible_right;
    let fits_below = cursor_pos.y + panel_logical_size.y <= visible_bottom;

    // println!("  Fits right: {}, fits below: {}", fits_right, fits_below);

    // Determine panel position based on available space (using logical coordinates)
    let panel_x = if fits_right {
        cursor_pos.x // Top-left or bottom-left at cursor
    } else {
        cursor_pos.x - panel_logical_size.x // Top-right or bottom-right at cursor
    };

    let panel_y = if fits_below {
        cursor_pos.y // Top-left or top-right at cursor
    } else {
        cursor_pos.y - panel_logical_size.y // Bottom-left or bottom-right at cursor
    };

    // Ensure the panel stays within visible area bounds (safety clamp, using logical coordinates)
    let final_x = panel_x
        .max(visible_left)
        .min(visible_right - panel_logical_size.x);
    let final_y = panel_y
        .max(visible_top)
        .min(visible_bottom - panel_logical_size.y);

    // Convert back to physical coordinates for Tauri API
    let physical_x = (final_x * scale_factor) as i32;
    let physical_y = (final_y * scale_factor) as i32;

    // Set panel position using Tauri API
    let position = Position::Physical(PhysicalPosition {
        x: physical_x,
        y: physical_y,
    });

    window
        .set_position(position)
        .map_err(|_| PositioningError::WindowHandleError)?;

    Ok(())
}

/// Get current cursor position in screen coordinates
fn get_cursor_position() -> Result<PhysicalPosition<f64>, PositioningError> {
    // Use Cocoa APIs to get cursor position
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSEvent, NSScreen};
        use cocoa::foundation::NSPoint;

        unsafe {
            let mouse_location: NSPoint = NSEvent::mouseLocation(std::ptr::null_mut());

            // Get the main screen to understand coordinate system
            let main_screen = NSScreen::mainScreen(std::ptr::null_mut());
            let main_screen_frame = NSScreen::frame(main_screen);

            // Cocoa uses bottom-left origin, but we want top-left origin
            // Convert from Cocoa coordinates to screen coordinates
            let screen_y = main_screen_frame.size.height - mouse_location.y;

            Ok(PhysicalPosition {
                x: mouse_location.x,
                y: screen_y,
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For non-macOS platforms, we'd need different implementation
        Err(PositioningError::MonitorNotFound)
    }
}

// Function to store the currently active application
#[cfg(target_os = "macos")]
pub fn store_previous_app() {
    println!("Storing previous app...");

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
                                println!("Stored previous app: {}", bundle_id);
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
    println!("Restoring previous app...");

    if let Ok(previous_app) = PREVIOUS_APP.lock() {
        if let Some(bundle_id) = previous_app.as_ref() {
            println!("Restoring focus to: {}", bundle_id);
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
                                println!("No running app found with bundle id: {}", bundle_id_owned);
                            }
                        } else {
                            println!("No running app array for bundle id: {}", bundle_id_owned);
                        }
                    });
                });
            }
        } else {
            println!("No previous app stored");
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn store_previous_app() {
    // No-op for non-macOS platforms
}

#[cfg(not(target_os = "macos"))]
pub fn restore_previous_app() {
    // No-op for non-macOS platforms
}
