const { invoke } = window.__TAURI__.core;
const python = window.__TAURI__.python.callFunction;

// DOM elements
const searchInput = document.getElementById('searchInput');
const emojiGrid = document.getElementById('emojiGrid');
const statusBar = document.getElementById('statusBar');

// Global variables
let gridEmojis = [];
let currentFilter = '';
let previousElementIndex = 0;

// Initialize app
document.addEventListener('DOMContentLoaded', async function () {
  console.log('EmojiQ initialized');

  await renderPanel();
  setupEventListeners();
});

async function renderPanel() {
  // Load emojis
  await loadEmojis();
  // Clear search input
  searchInput.value = "";
  // Focus search input
  searchInput.focus();
  // Update status bar
  handleSearchMouseOver();
}

// Setup event listeners
function setupEventListeners() {
  searchInput.addEventListener('input', handleSearch);
  searchInput.addEventListener('mouseover', handleSearchMouseOver);
  searchInput.addEventListener('keydown', handleSearchKeys);

  emojiGrid.addEventListener('keydown', handleNavigation);

  // Outside click
  // settingsModal.addEventListener('click', function (e) {
  //   if (e.target === settingsModal) {
  //     closeModal();
  //   }
  // });

  // Global shortcuts
  window.addEventListener('keydown', async function (e) {
    if (e.key === "Escape") {
      e.preventDefault();
      await invoke("hide_panel");
      await renderPanel();
    }
  });
}

// Load emojis from backend
async function loadEmojis(filter = '') {
  try {
    let emojis;

    emojis = await python("get_emojis", [filter]);

    // split emojis string by space
    emojis = emojis.split(' ');

    gridEmojis = emojis || [];
    renderEmojis();
  } catch (error) {
    console.error('Error loading emojis:', error);
    statusBar.textContent = 'Error loading emojis: ' + error.message;
  }
}

// Render emojis in grid
function renderEmojis() {
  emojiGrid.innerHTML = '';
  previousElementIndex = 0;

  if (!gridEmojis || gridEmojis.length === 0) {
    const noResults = document.createElement('div');
    noResults.className = 'no-results';
    noResults.textContent = 'No emojis found';
    emojiGrid.appendChild(noResults);
    return;
  }

  for (let i = 0; i < gridEmojis.length; i++) {
    const emoji = gridEmojis[i];
    const button = document.createElement('button');
    button.className = 'emoji-button';
    button.textContent = emoji;

    button.addEventListener('click', () => selectEmoji(emoji));
    button.addEventListener('mouseover', () => updateKeywords(emoji));
    emojiGrid.appendChild(button);
  }
}

// Handle search input
async function handleSearch(e) {
  const filter = e.target.value.trim();
  currentFilter = filter;

  // Debounce search
  clearTimeout(handleSearch.timeout);
  handleSearch.timeout = setTimeout(async () => {
    if (currentFilter === filter) {
      await loadEmojis(filter);
      if (gridEmojis.length > 0 && gridEmojis[0] !== '') {
        console.log(gridEmojis[0], gridEmojis.length);
        updateStatus(`Found ${gridEmojis.length} emoji${gridEmojis.length !== 1 ? 's' : ''}`);
      } else {
        updateStatus('No emojis found');
      }
    }
  }, 200);
}

function handleSearchMouseOver() {
  statusBar.className = 'status-bar-message';
  updateStatus('Click an emoji to paste it');
}

// Handle search keys
async function handleSearchKeys(e) {
  console.log("searchInput keydown:", e.key);
  switch (e.key) {
    case "ArrowDown":
      e.preventDefault();
      // focus on the first or previous emoji button
      const button = emojiGrid.querySelectorAll('.emoji-button')[previousElementIndex];
      if (button.textContent) {
        buttonFocus(button);
      }
      break;
    case "Enter":
      e.preventDefault();
      // click first emoji button
      const firstButton = emojiGrid.querySelector('.emoji-button');
      if (firstButton.textContent) {
        buttonFocus(firstButton);
        // give a chance to user to notice the focus before clicking
        await new Promise(resolve => setTimeout(resolve, 100));
        firstButton.click();
      }
      break;
  }
}

async function updateKeywords(emoji) {
  const keywords = await python("get_keywords", [emoji]);
  // split keywords by semicolon
  const keywordsArray = keywords.split(';');
  const description = keywordsArray[0];
  const keys = keywordsArray.slice(1).join(', ');
  console.log("buttonFocus:", emoji);
  console.log("keywords:", keywordsArray);
  updateStatus(`${description}\n${keys}`);
  statusBar.className = 'status-bar-keywords';
}

async function buttonFocus(button) {
  button.focus();
  const emoji = button.textContent;
  await updateKeywords(emoji);
}

// Handle keyboard navigation
function handleNavigation(e) {
  const buttons = emojiGrid.querySelectorAll('.emoji-button');
  if (buttons.length === 0) return;

  const focused = document.activeElement;
  let elementIndex = Array.from(buttons).indexOf(focused);
  let column = elementIndex % 10;
  let row = Math.floor(elementIndex / 10);
  console.log("Grid elementIndex:", elementIndex, "column:", column, "row:", row);
  console.log("Grid key:", e.key);

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      row++;
      elementIndex = row * 10 + column;
      if (elementIndex >= buttons.length) {
        elementIndex = buttons.length - 1;
      }
      buttonFocus(buttons[elementIndex]);
      break;
    case 'ArrowUp':
      e.preventDefault();
      if (row === 0) {
        searchInput.focus();
        previousElementIndex = elementIndex;
        break;
      }
      row--;
      elementIndex = row * 10 + column;
      buttonFocus(buttons[elementIndex]);
      break;
    case 'ArrowRight':
      e.preventDefault();
      if (column === 9) {
        row++;
        column = 0;
      } else {
        column++;
      }
      elementIndex = row * 10 + column;
      if (elementIndex < buttons.length) {
        buttonFocus(buttons[elementIndex]);
      }
      break;
    case 'ArrowLeft':
      e.preventDefault();
      if (column === 0 && row === 0) {
        break;
      }
      if (column === 0) {
        row--;
        column = 9;
      } else {
        column--;
      }
      elementIndex = row * 10 + column;
      buttonFocus(buttons[elementIndex]);
      break;
    case 'Enter':
      e.preventDefault();
      focused.click();
      break;
  }
}

// Select emoji
async function selectEmoji(emoji) {
  console.log("selectEmoji:", emoji);
  try {
    // // Call backend to increment usage and copy to clipboard
    // await window.wails.App.SelectEmoji(emoji.Character);
    // await window.wails.App.CopyEmojiToClipboard(emoji.Character);

    await python("increment_usage", [emoji]);
    await invoke("hide_panel");
    await invoke("type_emoji", { emoji: emoji });
    await renderPanel();

    // updateStatus(`Copied ${emoji} to clipboard! Press Cmd+V to paste.`);

    // // Reload emojis to update usage counts
    // setTimeout(() => loadEmojis(currentFilter), 100);

    // // Auto-hide after selection (would be handled by Wails window management)
    // setTimeout(() => {
    //   updateStatus('Click an emoji to copy it to clipboard');
    // }, 2000);

  } catch (error) {
    console.error('Error selecting emoji:', error);
    updateStatus('Error copying emoji to clipboard');
  }
}

// Update status bar
function updateStatus(message) {
  statusBar.textContent = message;
}
