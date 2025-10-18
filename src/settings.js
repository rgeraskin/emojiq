const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.shell;

let placeUnderMouseToggle;
let maxTopEmojisInput;
let emojiModePasteOnly;
let emojiModeCopyOnly;
let emojiModePasteAndCopy;

// Load settings on page load
async function loadSettings() {
  try {
    const settings = await invoke('get_settings');
    placeUnderMouseToggle.checked = settings.place_under_mouse;

    // Set the value
    const maxTopEmojisValue = settings.max_top_emojis !== undefined ? settings.max_top_emojis : 10;
    maxTopEmojisInput.value = maxTopEmojisValue;

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

    const parsedValue = parseInt(maxTopEmojisInput.value);
    const max_top_emojis = isNaN(parsedValue) ? 10 : parsedValue;

    const settings = {
      place_under_mouse: placeUnderMouseToggle.checked,
      emoji_mode: getSelectedEmojiMode(),
      max_top_emojis: max_top_emojis,
      window_width: currentSettings.window_width,
      window_height: currentSettings.window_height
    };
    await invoke('update_settings', { settings });
  } catch (error) {
    console.error('Failed to save settings:', error);
  }
}

// Increment/decrement handlers
function incrementValue() {
  const currentValue = parseInt(maxTopEmojisInput.value) || 0;
  const max = parseInt(maxTopEmojisInput.max) || 50;
  if (currentValue < max) {
    maxTopEmojisInput.value = currentValue + 1;
    saveSettings();
  }
}

function decrementValue() {
  const currentValue = parseInt(maxTopEmojisInput.value) || 0;
  const min = parseInt(maxTopEmojisInput.min) || 0;
  if (currentValue > min) {
    maxTopEmojisInput.value = currentValue - 1;
    saveSettings();
  }
}

// Setup event listeners
function setupEventListeners() {
  placeUnderMouseToggle.addEventListener('change', saveSettings);
  maxTopEmojisInput.addEventListener('change', saveSettings);
  emojiModePasteOnly.addEventListener('change', saveSettings);
  emojiModeCopyOnly.addEventListener('change', saveSettings);
  emojiModePasteAndCopy.addEventListener('change', saveSettings);

  // Custom spinner buttons
  const spinnerUp = document.getElementById('spinnerUp');
  const spinnerDown = document.getElementById('spinnerDown');

  spinnerUp.addEventListener('click', incrementValue);
  spinnerDown.addEventListener('click', decrementValue);

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
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', async () => {
  // Get DOM elements
  placeUnderMouseToggle = document.getElementById('placeUnderMouseToggle');
  maxTopEmojisInput = document.getElementById('maxTopEmojisInput');
  emojiModePasteOnly = document.getElementById('emojiModePasteOnly');
  emojiModeCopyOnly = document.getElementById('emojiModeCopyOnly');
  emojiModePasteAndCopy = document.getElementById('emojiModePasteAndCopy');

  // Load settings first
  await loadSettings();

  // Wait for next frame to ensure CSS is applied
  await new Promise(resolve => requestAnimationFrame(resolve));

  // Then setup event listeners
  setupEventListeners();
});

