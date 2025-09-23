# EmojiQ

Frustrated with every app reinventing its own emoji picker? Replace them with EmojiQ.
![logo](logo.png)

EmojiQ provides a floating panel with instant access to emojis through a global hotkey, featuring smart search, keyboard navigation, and seamless pasting.

Inspired by the excellent [qmoji](https://github.com/jaredly/qmoji) by Jared Forsyth.

## Features

- 🚀 **Instant access**: Open with `Cmd+Option+Space`
- 🔥 **Favorite emojis**: Top 10 most-used emojis appear first for quick access
- 🔍 **Fast search**: Find emojis by name, description, or keywords
- ⌨️ **Keyboard navigation**: Navigate with arrow keys; select with Enter/Space
- 📋 **Auto-paste**: Selected emojis are automatically pasted into your active application
- 🎯 **Smart positioning**: The panel appears under the mouse cursor
- 🎨 **Native design**: Built with the cross-platform [Tauri](https://tauri.app) framework
- 📦 **Small footprint**: Low resource usage, thanks to Rust!

![demo](demo.gif)

## Installation

```bash
brew tap rgeraskin/homebrew
brew install --cask emojiq
```

EmojiQ requires accessibility permissions to paste emojis into other applications. On first launch, click "Open System Settings" in the dialog and enable "EmojiQ" in the list.

You can grant or revoke this permission anytime in System Settings → Privacy & Security → Accessibility.

## Usage

1. **Open the picker**: Press Cmd+Option+Space
1. **Search**: Type to filter emojis by name or keywords
1. **Navigate**: Use arrow keys to move between emojis
1. **Hover**: Hover over an emoji to see its name and keywords
1. **Select**: Click an emoji or press Enter/Space to paste it
1. **Close**: Press Escape or click outside the panel

### Keyboard Shortcuts

- Cmd+Option+Space: Open/close the emoji panel
- ↑ ↓ ← →: Navigate between emojis
- Enter or Space: Select and paste
- Escape: Close the panel
- Home/End: Jump to first/last emoji
- Any character: Start typing to search

## Comparison with [qmoji](https://github.com/jaredly/qmoji)

It’s like qmoji, but with:
- keyboard navigation
- better focus management
- toggle the app with the same hotkey
- clear favorites logic: the more you use an emoji, the higher it appears
- more emojis and more keywords
- cross-platform by design (macOS supported today)
- MIT licensed
- alive and maintained :)

## Roadmap

Someday I might add:

- [ ] Settings with options like:
  - [ ] Number of recent emojis (currently hardcoded to 10)
  - [ ] Clear recent emoji stats (currently requires deleting the file manually)
  - [ ] Disable positioning under the mouse cursor
  - [ ] Custom hotkey configuration
  - [ ] Modes: Paste / Copy to clipboard / Paste & Copy to clipboard
- [ ] Linux support
- [ ] Windows support

Suggest a feature [here](https://github.com/rgeraskin/emojiq/issues/new)!

## Technical Details

### Architecture

- **Frontend**: Vanilla JavaScript with HTML/CSS
- **Backend**: Rust
- **UI Framework**: Cross-platform [Tauri](https://tauri.app) framework
- **Data**: JSON-based emoji database with metadata

## Development

### Building from Source

See tauri prerequisites [here](https://tauri.app/start/prerequisites/): Node.js, pnpm, and Rust toolchain.

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/emojiq.git
   cd emojiq
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Build and run (development):
   ```bash
   pnpm tauri dev
   ```

4. Build for production:
   ```bash
   pnpm tauri build
   ```

### Project Structure

```
emojiq/
├── src/                    # Frontend (HTML/CSS/JS)
│   ├── index.html
│   ├── main.js
│   └── styles.css
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── emoji_manager.rs
│   │   ├── panel.rs
│   │   ├── tray.rs
│   │   └── ...
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

Emoji stats are stored in `~/Library/Application Support/dev.rgeraskin.emojiq/ranks.json`.

### Contributing

Contributions are welcome!

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
