const { invoke } = window.__TAURI__.core;

// Configuration constants
const CONFIG = {
  EMOJI_BATCH_SIZE: 100,
  SEARCH_DEBOUNCE_MS: 150,
  BUTTON_FOCUS_DELAY_MS: 100,
};

// Polyfill for requestIdleCallback if not available (Tauri webview compatibility)
const requestIdleCallback = window.requestIdleCallback || function (callback) {
  return setTimeout(callback, 16); // ~60fps fallback
};

// DOM elements
const searchInput = document.getElementById('searchInput');
const emojiGrid = document.getElementById('emojiGrid');
const statusBar = document.getElementById('statusBar');

// Global variables
let gridEmojis = [];
let currentFilter = '';
let previousElementIndex = 0;
let emojiCache = new Map(); // Cache for rendered emoji buttons
let currentBatch = 0;

// Initialize app
document.addEventListener('DOMContentLoaded', async function () {
  console.log('EmojiQ initialized');

  await renderPanel();
  setupEventListeners();
  setupWindowResizeHandler();
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

// Setup event listeners with delegation
function setupEventListeners() {
  console.log('Setting up event listeners...');

  searchInput.addEventListener('input', handleSearch);
  searchInput.addEventListener('mouseover', handleSearchMouseOver);
  searchInput.addEventListener('keydown', handleSearchKeys);
  searchInput.addEventListener('click', async function (e) {
    e.preventDefault();
    previousElementIndex = 0;
  });

  // Use event delegation for emoji grid
  emojiGrid.addEventListener('keydown', handleGridNavigation);
  emojiGrid.addEventListener('click', handleEmojiClick);
  emojiGrid.addEventListener('mouseover', handleEmojiMouseOver);

  // Global shortcuts
  window.addEventListener('keydown', async function (e) {
    // console.log('Key pressed:', e.key);
    if (e.key === "Escape") {
      // console.log('ESC key detected, hiding panel');
      e.preventDefault();
      await invoke("hide_panel");
      await renderPanel();
      return;
    }

    // Cmd+, to open settings
    if (e.key === ',' && e.metaKey && !e.shiftKey && !e.altKey && !e.ctrlKey) {
      e.preventDefault();
      await invoke("open_settings");
      return;
    }

    // Focus search input when typing any character (but not special keys)
    const isTypingCharacter = e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey;
    const isSearchFocused = document.activeElement === searchInput;

    if (isTypingCharacter && !isSearchFocused) {
      // Focus search input and let the character be typed
      searchInput.focus();
      // Don't prevent default - let the character be typed in the search input
    }
  });
}

// Event delegation handlers
function handleEmojiClick(e) {
  if (e.target.classList.contains('emoji-button')) {
    const emoji = e.target.dataset.emoji || e.target.textContent;
    selectEmoji(emoji);
  }
}

function handleEmojiMouseOver(e) {
  if (e.target.classList.contains('emoji-button')) {
    const emoji = e.target.dataset.emoji || e.target.textContent;
    updateKeywords(emoji);
  }
}

// Load emojis from backend
async function loadEmojis(filter = '') {
  try {
    // Show loading state
    updateStatus('Loading emojis...');

    const emojis = await invoke("get_emojis", { filterWord: filter });

    // Validate response
    if (!Array.isArray(emojis)) {
      throw new Error('Invalid response format from backend');
    }

    // Trim emojis to avoid empty strings
    gridEmojis = emojis.filter(emoji => emoji.trim() !== '');

    renderEmojis();

    // Update status with results
    if (gridEmojis.length > 0) {
      updateStatus(`Found ${gridEmojis.length} emoji${gridEmojis.length !== 1 ? 's' : ''}`);
    } else {
      updateStatus('Need mooooore emojis 😜');
    }
  } catch (error) {
    console.error('Error loading emojis:', error);
    gridEmojis = [];
    renderEmojis();
    updateStatus(`Error: ${error.message || 'Failed to load emojis'}`);
  }
}

// Render emojis in grid
function renderEmojis() {
  // Clear previous content efficiently
  const fragment = document.createDocumentFragment();
  emojiGrid.innerHTML = '';
  previousElementIndex = 0;

  if (!gridEmojis || gridEmojis.length === 0) {
    const noResults = document.createElement('div');
    noResults.className = 'no-results';
    noResults.textContent = 'No emojis found';
    emojiGrid.appendChild(noResults);
    return;
  }

  // Render emojis in batches to improve performance
  const endIndex = Math.min(gridEmojis.length, CONFIG.EMOJI_BATCH_SIZE);

  for (let i = 0; i < endIndex; i++) {
    const emoji = gridEmojis[i];
    if (!emoji) continue; // Skip empty emojis

    const button = createEmojiButton(emoji);
    fragment.appendChild(button);
  }

  emojiGrid.appendChild(fragment);

  // Load more emojis if needed (lazy loading)
  if (gridEmojis.length > CONFIG.EMOJI_BATCH_SIZE) {
    requestIdleCallback(() => loadMoreEmojis(endIndex));
  }
}

// Create emoji button
function createEmojiButton(emoji) {
  const button = document.createElement('button');
  button.className = 'emoji-button';
  button.textContent = emoji;
  button.dataset.emoji = emoji; // Store emoji in data attribute for reliability
  return button;
}

// Load more emojis lazily
function loadMoreEmojis(startIndex) {
  const fragment = document.createDocumentFragment();
  const endIndex = Math.min(gridEmojis.length, startIndex + CONFIG.EMOJI_BATCH_SIZE);

  for (let i = startIndex; i < endIndex; i++) {
    const emoji = gridEmojis[i];
    if (!emoji) continue;

    const button = createEmojiButton(emoji);
    fragment.appendChild(button);
  }

  emojiGrid.appendChild(fragment);

  if (endIndex < gridEmojis.length) {
    requestIdleCallback(() => loadMoreEmojis(endIndex));
  }
}

// Handle search input with debouncing
let searchController; // AbortController for canceling requests

async function handleSearch(e) {
  const filter = e.target.value.trim();
  currentFilter = filter;

  // Cancel previous request if still pending
  if (searchController) {
    searchController.abort();
  }

  // Create new controller for this request
  searchController = new AbortController();

  // Debounce search with delay
  clearTimeout(handleSearch.timeout);
  handleSearch.timeout = setTimeout(async () => {
    if (currentFilter === filter && !searchController.signal.aborted) {
      try {
        await loadEmojis(filter);
      } catch (error) {
        if (error.name !== 'AbortError') {
          console.error('Search error:', error);
        }
      }
    }
  }, CONFIG.SEARCH_DEBOUNCE_MS);
}

function handleSearchMouseOver() {
  statusBar.className = 'status-bar-message';
  updateStatus('Click an emoji to paste it');
}

// Handle search keys
async function handleSearchKeys(e) {
  // console.log("searchInput keydown:", e.key);
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
        await new Promise(resolve => setTimeout(resolve, CONFIG.BUTTON_FOCUS_DELAY_MS));
        firstButton.click();
      }
      break;
  }
}

async function updateKeywords(emoji) {
  const keywords = await invoke("get_keywords", { emoji: emoji });
  const description = keywords[0] || '';
  const keys = keywords.slice(1).join(', ');
  // console.log("buttonFocus:", emoji);
  // console.log("keywords:", keywords);
  updateStatus(`${description}\n${keys}`);
  statusBar.className = 'status-bar-keywords';
}

async function buttonFocus(button) {
  button.focus();
  const emoji = button.textContent;
  await updateKeywords(emoji);
}

// Get dynamic number of columns in the grid
function getGridColumns() {
  const buttons = emojiGrid.querySelectorAll('.emoji-button');
  if (buttons.length < 2) return 1;

  // Calculate columns by comparing Y positions of first two buttons
  const firstButtonRect = buttons[0].getBoundingClientRect();
  const secondButtonRect = buttons[1].getBoundingClientRect();

  // If they're on the same row, count how many are on this row
  if (Math.abs(firstButtonRect.top - secondButtonRect.top) < 5) {
    let cols = 1;
    for (let i = 1; i < buttons.length; i++) {
      const rect = buttons[i].getBoundingClientRect();
      if (Math.abs(rect.top - firstButtonRect.top) < 5) {
        cols++;
      } else {
        break;
      }
    }
    return cols;
  }

  return 1;
}

// Handle keyboard navigation in emoji grid
function handleGridNavigation(e) {
  const buttons = emojiGrid.querySelectorAll('.emoji-button');
  if (buttons.length === 0) return;

  const focused = document.activeElement;
  if (!focused.classList.contains('emoji-button')) return;

  const gridColumns = getGridColumns();
  let elementIndex = Array.from(buttons).indexOf(focused);
  let column = elementIndex % gridColumns;
  let row = Math.floor(elementIndex / gridColumns);
  const totalRows = Math.ceil(buttons.length / gridColumns);

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      if (row < totalRows - 1) {
        row++;
        elementIndex = Math.min(row * gridColumns + column, buttons.length - 1);
        buttonFocus(buttons[elementIndex]);
      }
      break;
    case 'ArrowUp':
      e.preventDefault();
      if (row === 0) {
        searchInput.focus();
        previousElementIndex = elementIndex;
      } else {
        row--;
        elementIndex = row * gridColumns + column;
        buttonFocus(buttons[elementIndex]);
      }
      break;
    case 'ArrowRight':
      e.preventDefault();
      if (elementIndex < buttons.length - 1) {
        elementIndex++;
        buttonFocus(buttons[elementIndex]);
      }
      break;
    case 'ArrowLeft':
      e.preventDefault();
      if (elementIndex > 0) {
        elementIndex--;
        buttonFocus(buttons[elementIndex]);
      }
      break;
    case 'Home':
      e.preventDefault();
      buttonFocus(buttons[0]);
      break;
    case 'End':
      e.preventDefault();
      buttonFocus(buttons[buttons.length - 1]);
      break;
    case 'Enter':
    case ' ':
      e.preventDefault();
      focused.click();
      break;
  }
}

// Select emoji
async function selectEmoji(emoji) {
  console.log("selectEmoji:", emoji);
  try {
    await invoke("hide_panel");
    await invoke("type_emoji", { emoji: emoji });
    await invoke("increment_usage", { emoji: emoji });
  } catch (error) {
    console.error('Error selecting emoji:', error);
    updateStatus('Error pasting emoji o_0');
  }
  await renderPanel();
}

// Update status bar
function updateStatus(message) {
  statusBar.textContent = message;
}

// Setup window resize handler to save size
function setupWindowResizeHandler() {
  let resizeTimeout;

  // Save window size when it changes
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(async () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      try {
        await invoke('save_window_size', { width, height });
      } catch (error) {
        console.error('Failed to save window size:', error);
      }
    }, 500);
  });

  // Implement custom resize handle for NSPanel
  const resizeHandle = document.getElementById('resizeHandle');
  if (resizeHandle) {
    let isResizing = false;
    let startX, startY, startWidth, startHeight;

    // Import getCurrentWindow
    const { getCurrentWindow } = window.__TAURI__.window;
    const currentWindow = getCurrentWindow();

    resizeHandle.addEventListener('mousedown', (e) => {
      e.preventDefault();
      e.stopPropagation();
      isResizing = true;

      startX = e.clientX;
      startY = e.clientY;
      startWidth = window.innerWidth;
      startHeight = window.innerHeight;

      const handleMouseMove = (moveEvent) => {
        if (!isResizing) return;

        const deltaX = moveEvent.clientX - startX;
        const deltaY = moveEvent.clientY - startY;

        const newWidth = Math.max(120, startWidth + deltaX);
        const newHeight = Math.max(120, startHeight + deltaY);

        currentWindow.setSize({ type: 'Logical', width: newWidth, height: newHeight })
          .catch(error => console.error('Failed to resize window:', error));
      };

      const handleMouseUp = () => {
        isResizing = false;
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
      };

      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    });
  }
}
