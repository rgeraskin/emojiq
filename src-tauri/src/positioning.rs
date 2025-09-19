use tauri::{PhysicalPosition, Position};

#[cfg(target_os = "macos")]
use cocoa::appkit::NSScreen;
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSPoint, NSRect};

#[cfg(not(target_os = "macos"))]
use monitor::get_monitor_with_cursor;

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

                println!("Found screen with cursor:");
                println!(
                    "  Full frame: {}x{} at ({}, {})",
                    screen_frame.size.width,
                    screen_frame.size.height,
                    screen_frame.origin.x,
                    screen_frame.origin.y
                );
                println!(
                    "  Visible frame (Cocoa): {}x{} at ({}, {})",
                    screen_visible_frame.size.width,
                    screen_visible_frame.size.height,
                    screen_visible_frame.origin.x,
                    screen_visible_frame.origin.y
                );
                println!(
                    "  Visible frame (converted): {}x{} at ({}, {})",
                    visible_frame.size.width,
                    visible_frame.size.height,
                    visible_frame.origin.x,
                    visible_frame.origin.y
                );
                println!("  Scale factor: {}", scale_factor);

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

    println!("Debug positioning (dock-aware):");
    println!("  Cursor: ({}, {})", cursor_pos.x, cursor_pos.y);
    println!(
        "  Panel size (physical): {}x{}",
        panel_size.width, panel_size.height
    );
    println!(
        "  Panel size (logical): {}x{}",
        panel_logical_size.x, panel_logical_size.y
    );
    println!(
        "  Visible area: {}x{} at ({}, {}), scale: {}",
        visible_area.width(),
        visible_area.height(),
        visible_area.left(),
        visible_area.top(),
        scale_factor
    );

    // Calculate visible area bounds in logical coordinates (to match cursor)
    let visible_left = visible_area.left();
    let visible_top = visible_area.top();
    let visible_right = visible_left + visible_area.width();
    let visible_bottom = visible_top + visible_area.height();

    // Calculate if panel fits in each direction from cursor (using logical coordinates)
    let fits_right = cursor_pos.x + panel_logical_size.x <= visible_right;
    let fits_below = cursor_pos.y + panel_logical_size.y <= visible_bottom;

    println!("  Fits right: {}, fits below: {}", fits_right, fits_below);

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

    println!("  Final position (logical): ({}, {})", final_x, final_y);

    // Convert back to physical coordinates for Tauri API
    let physical_x = (final_x * scale_factor) as i32;
    let physical_y = (final_y * scale_factor) as i32;

    println!(
        "  Final position (physical): ({}, {})",
        physical_x, physical_y
    );

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
