const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const placeUnderMouseToggle = document.getElementById('placeUnderMouseToggle');
const statusMessage = document.getElementById('statusMessage');

// Load settings on page load
async function loadSettings() {
  try {
    const settings = await invoke('get_settings');
    placeUnderMouseToggle.checked = settings.place_under_mouse;
  } catch (error) {
    console.error('Failed to load settings:', error);
    showStatus('Failed to load settings', 'error');
  }
}

// Save settings when changed
async function saveSettings() {
  try {
    const settings = {
      place_under_mouse: placeUnderMouseToggle.checked
    };
    await invoke('update_settings', { settings });
    showStatus('Settings saved successfully', 'success');
  } catch (error) {
    console.error('Failed to save settings:', error);
    showStatus('Failed to save settings', 'error');
  }
}

// Show status message
function showStatus(message, type) {
  statusMessage.textContent = message;
  statusMessage.className = `status-message ${type} show`;
  setTimeout(() => {
    statusMessage.classList.remove('show');
  }, 3000);
}

// Event listeners
placeUnderMouseToggle.addEventListener('change', saveSettings);

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

// Load settings on page load
loadSettings();

