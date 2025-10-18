const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.shell;

const placeUnderMouseToggle = document.getElementById('placeUnderMouseToggle');
const emojiModePasteOnly = document.getElementById('emojiModePasteOnly');
const emojiModeCopyOnly = document.getElementById('emojiModeCopyOnly');
const emojiModePasteAndCopy = document.getElementById('emojiModePasteAndCopy');

// Load settings on page load
async function loadSettings() {
  try {
    const settings = await invoke('get_settings');
    placeUnderMouseToggle.checked = settings.place_under_mouse;

    // Set the appropriate radio button based on emoji_mode
    const emojiMode = settings.emoji_mode || 'paste_only';
    switch (emojiMode) {
      case 'paste_only':
        emojiModePasteOnly.checked = true;
        break;
      case 'copy_only':
        emojiModeCopyOnly.checked = true;
        break;
      case 'paste_and_copy':
        emojiModePasteAndCopy.checked = true;
        break;
      default:
        emojiModePasteOnly.checked = true;
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
  }
}

// Get selected emoji mode
function getSelectedEmojiMode() {
  if (emojiModePasteOnly.checked) return 'paste_only';
  if (emojiModeCopyOnly.checked) return 'copy_only';
  if (emojiModePasteAndCopy.checked) return 'paste_and_copy';
  return 'paste_only'; // default
}

// Save settings when changed
async function saveSettings() {
  try {
    // Get current settings first to preserve window size
    const currentSettings = await invoke('get_settings');

    const settings = {
      place_under_mouse: placeUnderMouseToggle.checked,
      emoji_mode: getSelectedEmojiMode(),
      window_width: currentSettings.window_width,
      window_height: currentSettings.window_height
    };
    await invoke('update_settings', { settings });
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

// Event listeners
placeUnderMouseToggle.addEventListener('change', saveSettings);
emojiModePasteOnly.addEventListener('change', saveSettings);
emojiModeCopyOnly.addEventListener('change', saveSettings);
emojiModePasteAndCopy.addEventListener('change', saveSettings);

// ESC key to close settings window
window.addEventListener('keydown', async (e) => {
  if (e.key === 'Escape') {
    e.preventDefault();
    try {
      await getCurrentWindow().close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  }
});

// Handle external links
document.addEventListener('click', async (e) => {
  const target = e.target.closest('a');
  if (target && target.href && target.href.startsWith('http')) {
    e.preventDefault();
    try {
      await open(target.href);
    } catch (error) {
      console.error('Failed to open link:', error);
    }
  }
});

// Load settings on page load
loadSettings();

