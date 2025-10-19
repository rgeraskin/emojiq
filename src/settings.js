const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { open } = window.__TAURI__.shell;
const { ask } = window.__TAURI__.dialog;

let hotkeyInput;
let hotkeyReset;
let placeUnderMouseToggle;
let maxTopEmojisInput;
let emojiModePasteOnly;
let emojiModeCopyOnly;
let emojiModePasteAndCopy;
let scaleFactorSlider;
let scaleValue;
let resetRanksButton;

// Load settings on page load
async function loadSettings() {
  try {
    const settings = await invoke('get_settings');

    // Set hotkey
    const globalHotkey = settings.global_hotkey || 'Cmd+Option+Space';
    hotkeyInput.value = globalHotkey;

    placeUnderMouseToggle.checked = settings.place_under_mouse;

    // Set the value
    const maxTopEmojisValue = settings.max_top_emojis !== undefined ? settings.max_top_emojis : 10;
    maxTopEmojisInput.value = maxTopEmojisValue;

    // Set scale factor
    const scaleFactor = settings.scale_factor !== undefined ? settings.scale_factor : 1.0;
    scaleFactorSlider.value = scaleFactor;
    updateScaleDisplay(scaleFactor);

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

// Update scale factor display
function updateScaleDisplay(value) {
  const percentage = Math.round(value * 100);
  scaleValue.textContent = `${percentage}%`;
}

// Save settings when changed
async function saveSettings() {
  try {
    // Get current settings first to preserve window size
    const currentSettings = await invoke('get_settings');

    const parsedValue = parseInt(maxTopEmojisInput.value);
    const max_top_emojis = isNaN(parsedValue) ? 10 : parsedValue;

    const parsedScaleFactor = parseFloat(scaleFactorSlider.value);
    const scale_factor = isNaN(parsedScaleFactor) ? 1.0 : parsedScaleFactor;

    const settings = {
      global_hotkey: hotkeyInput.value || 'Cmd+Option+Space',
      place_under_mouse: placeUnderMouseToggle.checked,
      emoji_mode: getSelectedEmojiMode(),
      max_top_emojis: max_top_emojis,
      scale_factor: scale_factor,
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

// Hotkey capture functionality
let isCapturingHotkey = false;
let hotkeyCaptured = false;

function captureHotkey(event) {
  if (!isCapturingHotkey) return;

  event.preventDefault();
  event.stopPropagation();

  // Build the key combination string
  const parts = [];
  let hasNonModifierKey = false;

  // Map to macOS-style names
  if (event.metaKey) parts.push('Cmd');
  if (event.ctrlKey) parts.push('Ctrl');
  if (event.altKey) parts.push('Option');
  if (event.shiftKey) parts.push('Shift');

  // Use event.code for physical key (not affected by shift/modifiers)
  const code = event.code;

  // Skip if it's just a modifier key
  if (!['MetaLeft', 'MetaRight', 'ControlLeft', 'ControlRight',
    'AltLeft', 'AltRight', 'ShiftLeft', 'ShiftRight'].includes(code)) {

    let keyName = codeToKeyName(code);
    if (keyName) {
      parts.push(keyName);
      hasNonModifierKey = true;
    }
  }

  // Need at least one modifier and one non-modifier key
  if (parts.length >= 2 && hasNonModifierKey) {
    hotkeyInput.value = parts.join('+');
    hotkeyCaptured = true;
    isCapturingHotkey = false;
    saveSettings();
    hotkeyInput.blur();
  }
}

// Convert JavaScript event.code to our key name format
function codeToKeyName(code) {
  // Letters (KeyA -> A)
  if (code.startsWith('Key')) {
    return code.substring(3).toUpperCase();
  }

  // Numbers (Digit0 -> 0)
  if (code.startsWith('Digit')) {
    return code.substring(5);
  }

  // Function keys (F1-F12)
  if (code.match(/^F\d+$/)) {
    return code;
  }

  // Special keys mapping
  const specialKeys = {
    'Space': 'Space',
    'Enter': 'Enter',
    'Tab': 'Tab',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Escape': 'Escape',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    'ArrowUp': 'ArrowUp',
    'ArrowDown': 'ArrowDown',
    'ArrowLeft': 'ArrowLeft',
    'ArrowRight': 'ArrowRight',
    'Minus': 'Minus',
    'Equal': 'Equal',
    'BracketLeft': 'BracketLeft',
    'BracketRight': 'BracketRight',
    'Backslash': 'Backslash',
    'Semicolon': 'Semicolon',
    'Quote': 'Quote',
    'Comma': 'Comma',
    'Period': 'Period',
    'Slash': 'Slash',
    'Backquote': 'Backquote',
  };

  return specialKeys[code] || null;
}

function startHotkeyCapture() {
  isCapturingHotkey = true;
  hotkeyCaptured = false;
  hotkeyInput.value = 'Press keys...';
  hotkeyInput.classList.add('capturing');
}

function stopHotkeyCapture() {
  isCapturingHotkey = false;
  hotkeyInput.classList.remove('capturing');

  // Only restore previous value if user didn't capture a new hotkey
  if (!hotkeyCaptured) {
    loadSettings();
  }

  // Reset flag
  hotkeyCaptured = false;
}

function resetHotkey() {
  hotkeyInput.value = 'Cmd+Option+Space';
  saveSettings();
}

// Reset emoji ranks handler
async function handleResetRanks() {
  console.log('Reset button clicked');

  try {
    const confirmed = await ask('Are you sure you want to reset all emoji usage statistics? This cannot be undone.', {
      title: 'Reset Emoji Ranks',
      type: 'warning',
      okLabel: 'Reset',
      cancelLabel: 'Cancel'
    });

    if (!confirmed) {
      console.log('User cancelled reset');
      return;
    }

    console.log('Starting reset...');
    resetRanksButton.disabled = true;
    resetRanksButton.textContent = 'Resetting...';

    await invoke('reset_emoji_ranks');
    console.log('Reset successful');

    resetRanksButton.textContent = 'Reset Complete!';
    setTimeout(() => {
      resetRanksButton.textContent = 'Reset Emoji Ranks';
      resetRanksButton.disabled = false;
    }, 2000);
  } catch (error) {
    console.error('Failed to reset emoji ranks:', error);
    alert('Failed to reset emoji ranks: ' + error);
    resetRanksButton.textContent = 'Reset Emoji Ranks';
    resetRanksButton.disabled = false;
  }
}

// Setup event listeners
function setupEventListeners() {
  // Hotkey input
  hotkeyInput.addEventListener('focus', startHotkeyCapture);
  hotkeyInput.addEventListener('blur', stopHotkeyCapture);
  hotkeyInput.addEventListener('keydown', captureHotkey);
  hotkeyReset.addEventListener('click', resetHotkey);

  placeUnderMouseToggle.addEventListener('change', saveSettings);
  maxTopEmojisInput.addEventListener('change', saveSettings);
  emojiModePasteOnly.addEventListener('change', saveSettings);
  emojiModeCopyOnly.addEventListener('change', saveSettings);
  emojiModePasteAndCopy.addEventListener('change', saveSettings);

  // Scale factor slider
  scaleFactorSlider.addEventListener('input', (e) => {
    updateScaleDisplay(parseFloat(e.target.value));
  });
  scaleFactorSlider.addEventListener('change', saveSettings);

  // Custom spinner buttons
  const spinnerUp = document.getElementById('spinnerUp');
  const spinnerDown = document.getElementById('spinnerDown');

  spinnerUp.addEventListener('click', incrementValue);
  spinnerDown.addEventListener('click', decrementValue);

  // Reset ranks button
  resetRanksButton.addEventListener('click', handleResetRanks);

  // ESC key to close settings window (but not when capturing hotkey)
  window.addEventListener('keydown', async (e) => {
    if (e.key === 'Escape' && !isCapturingHotkey) {
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
  hotkeyInput = document.getElementById('hotkeyInput');
  hotkeyReset = document.getElementById('hotkeyReset');
  placeUnderMouseToggle = document.getElementById('placeUnderMouseToggle');
  maxTopEmojisInput = document.getElementById('maxTopEmojisInput');
  emojiModePasteOnly = document.getElementById('emojiModePasteOnly');
  emojiModeCopyOnly = document.getElementById('emojiModeCopyOnly');
  emojiModePasteAndCopy = document.getElementById('emojiModePasteAndCopy');
  scaleFactorSlider = document.getElementById('scaleFactorSlider');
  scaleValue = document.getElementById('scaleValue');
  resetRanksButton = document.getElementById('resetRanksButton');

  // Load settings first
  await loadSettings();

  // Wait for next frame to ensure CSS is applied
  await new Promise(resolve => requestAnimationFrame(resolve));

  // Then setup event listeners
  setupEventListeners();

});

