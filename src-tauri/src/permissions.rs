use crate::errors::EmojiError;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri_plugin_macos_permissions::{
    check_accessibility_permission, request_accessibility_permission,
};

// Global state for permission caching
static PERMISSION_GRANTED: AtomicBool = AtomicBool::new(false);
static PERMISSION_CHECKED: AtomicBool = AtomicBool::new(false);

/// Check and cache accessibility permission status
pub async fn ensure_accessibility_permission() -> Result<(), EmojiError> {
    // If we've already checked and have permission, return early
    if PERMISSION_CHECKED.load(Ordering::Relaxed) && PERMISSION_GRANTED.load(Ordering::Relaxed) {
        return Ok(());
    }

    // Check permission status
    let authorized = check_accessibility_permission().await;

    if authorized {
        // Cache the positive result
        PERMISSION_GRANTED.store(true, Ordering::Relaxed);
        PERMISSION_CHECKED.store(true, Ordering::Relaxed);
        log::info!("Accessibility permission already granted");
        Ok(())
    } else {
        log::info!("Requesting accessibility permission");
        request_accessibility_permission().await;

        // Re-check after request
        let authorized_after_request = check_accessibility_permission().await;
        PERMISSION_GRANTED.store(authorized_after_request, Ordering::Relaxed);
        PERMISSION_CHECKED.store(true, Ordering::Relaxed);

        if authorized_after_request {
            Ok(())
        } else {
            Err(EmojiError::Permission("Accessibility permission denied. Please grant permission in System Preferences > Security & Privacy > Privacy > Accessibility".to_string()))
        }
    }
}

/// Reset the permission cache (useful for testing or if permissions change)
#[allow(dead_code)]
pub fn reset_permission_cache() {
    PERMISSION_GRANTED.store(false, Ordering::Relaxed);
    PERMISSION_CHECKED.store(false, Ordering::Relaxed);
}
