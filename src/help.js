async function closeHelp() {
  try {
    await (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke
      ? window.__TAURI__.core.invoke('close_help')
      : Promise.resolve());
  } catch (_) { }
}

// Close window on Escape
document.addEventListener('keydown', (e) => {
  const isEscape = e.key === 'Escape';
  const isEnter = e.key === 'Enter';
  const isSpace = e.key === ' ' || e.code === 'Space' || e.key === 'Spacebar';
  if (isEscape || isEnter || isSpace) {
    e.preventDefault();
    closeHelp();
  }
});

// Close window on button click
document.addEventListener('DOMContentLoaded', () => {
  const btn = document.getElementById('helpCloseBtn');
  if (!btn) return;
  btn.addEventListener('click', () => {
    closeHelp();
  });
});
