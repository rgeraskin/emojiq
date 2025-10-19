use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

/// Parse a hotkey string (e.g., "Cmd+Option+Space") into Tauri's Shortcut format
pub fn parse_hotkey(hotkey_str: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    if parts.is_empty() {
        return Err("Hotkey string is empty".to_string());
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        let part = part.trim();
        match part {
            "Cmd" | "Command" | "Super" => modifiers |= Modifiers::SUPER,
            "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
            "Option" | "Alt" => modifiers |= Modifiers::ALT,
            "Shift" => modifiers |= Modifiers::SHIFT,
            // Parse the key code
            key => {
                if key_code.is_some() {
                    return Err("Multiple key codes specified".to_string());
                }
                key_code = Some(parse_key_code(key)?);
            }
        }
    }

    let key_code = key_code.ok_or_else(|| "No key code specified".to_string())?;

    // Modifiers are optional in Shortcut::new, but we'll use Some() if we have any
    let modifiers = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    };

    Ok(Shortcut::new(modifiers, key_code))
}

/// Parse a key string into a Code
fn parse_key_code(key: &str) -> Result<Code, String> {
    match key.to_uppercase().as_str() {
        // Letters
        "A" => Ok(Code::KeyA),
        "B" => Ok(Code::KeyB),
        "C" => Ok(Code::KeyC),
        "D" => Ok(Code::KeyD),
        "E" => Ok(Code::KeyE),
        "F" => Ok(Code::KeyF),
        "G" => Ok(Code::KeyG),
        "H" => Ok(Code::KeyH),
        "I" => Ok(Code::KeyI),
        "J" => Ok(Code::KeyJ),
        "K" => Ok(Code::KeyK),
        "L" => Ok(Code::KeyL),
        "M" => Ok(Code::KeyM),
        "N" => Ok(Code::KeyN),
        "O" => Ok(Code::KeyO),
        "P" => Ok(Code::KeyP),
        "Q" => Ok(Code::KeyQ),
        "R" => Ok(Code::KeyR),
        "S" => Ok(Code::KeyS),
        "T" => Ok(Code::KeyT),
        "U" => Ok(Code::KeyU),
        "V" => Ok(Code::KeyV),
        "W" => Ok(Code::KeyW),
        "X" => Ok(Code::KeyX),
        "Y" => Ok(Code::KeyY),
        "Z" => Ok(Code::KeyZ),

        // Numbers
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),

        // Function keys
        "F1" => Ok(Code::F1),
        "F2" => Ok(Code::F2),
        "F3" => Ok(Code::F3),
        "F4" => Ok(Code::F4),
        "F5" => Ok(Code::F5),
        "F6" => Ok(Code::F6),
        "F7" => Ok(Code::F7),
        "F8" => Ok(Code::F8),
        "F9" => Ok(Code::F9),
        "F10" => Ok(Code::F10),
        "F11" => Ok(Code::F11),
        "F12" => Ok(Code::F12),

        // Special keys
        "SPACE" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "TAB" => Ok(Code::Tab),
        "BACKSPACE" => Ok(Code::Backspace),
        "ESCAPE" | "ESC" => Ok(Code::Escape),
        "DELETE" | "DEL" => Ok(Code::Delete),
        "HOME" => Ok(Code::Home),
        "END" => Ok(Code::End),
        "PAGEUP" => Ok(Code::PageUp),
        "PAGEDOWN" => Ok(Code::PageDown),
        "ARROWUP" | "UP" => Ok(Code::ArrowUp),
        "ARROWDOWN" | "DOWN" => Ok(Code::ArrowDown),
        "ARROWLEFT" | "LEFT" => Ok(Code::ArrowLeft),
        "ARROWRIGHT" | "RIGHT" => Ok(Code::ArrowRight),

        // Punctuation
        "MINUS" | "-" => Ok(Code::Minus),
        "EQUAL" | "=" => Ok(Code::Equal),
        "BRACKETLEFT" | "[" => Ok(Code::BracketLeft),
        "BRACKETRIGHT" | "]" => Ok(Code::BracketRight),
        "BACKSLASH" | "\\" => Ok(Code::Backslash),
        "SEMICOLON" | ";" => Ok(Code::Semicolon),
        "QUOTE" | "'" | "\"" => Ok(Code::Quote),
        "COMMA" | "," => Ok(Code::Comma),
        "PERIOD" | "." => Ok(Code::Period),
        "SLASH" | "/" => Ok(Code::Slash),
        "BACKQUOTE" | "`" => Ok(Code::Backquote),

        _ => Err(format!("Unknown key: {}", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cmd_option_space() {
        let result = parse_hotkey("Cmd+Option+Space");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_ctrl_shift_a() {
        let result = parse_hotkey("Ctrl+Shift+A");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_cmd_shift_backslash() {
        let result = parse_hotkey("Cmd+Shift+\\");
        assert!(result.is_ok(), "Failed to parse Cmd+Shift+\\");
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse_hotkey("InvalidKey");
        assert!(result.is_err());
    }
}
