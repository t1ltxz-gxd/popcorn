use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use rdev::Key as RdevKey;

/// Exploded keyboard shortcut of the "Ctrl+Alt+F3" type.
/// Modifiers (in any order, case is not important):
/// ctrl/control, alt, shift, win/super/meta/cmd.
#[derive(Debug, Clone, PartialEq)]
pub struct Hotkey {
	pub ctrl: bool,
	pub alt: bool,
	pub shift: bool,
	pub win: bool,
	pub key: String,
}

pub fn parse(spec: &str) -> Option<Hotkey> {
	let mut ctrl = false;
	let mut alt = false;
	let mut shift = false;
	let mut win = false;
	let mut key: Option<String> = None;

	for part in spec.split('+') {
		let part = part.trim();
		if part.is_empty() {
			continue;
		}
		match part.to_ascii_lowercase().as_str() {
			"ctrl" | "control" => ctrl = true,
			"alt" => alt = true,
			"shift" => shift = true,
			"win" | "super" | "meta" | "cmd" => win = true,
			other => {
				//more than one "non-modifying" part - the config is broken
				if key.is_some() {
					return None;
				}
				key = Some(other.to_string());
			}
		}
	}

	Some(Hotkey {
		ctrl,
		alt,
		shift,
		win,
		key: key?,
	})
}

// Current state of modifiers — updated on each
// key press/release.
static CTRL: AtomicBool = AtomicBool::new(false);
static ALT: AtomicBool = AtomicBool::new(false);
static SHIFT: AtomicBool = AtomicBool::new(false);
static WIN: AtomicBool = AtomicBool::new(false);

pub fn update_modifier_state(key: RdevKey, pressed: bool) {
	match key {
		RdevKey::ControlLeft | RdevKey::ControlRight => CTRL.store(pressed, Ordering::Relaxed),
		RdevKey::Alt | RdevKey::AltGr => ALT.store(pressed, Ordering::Relaxed),
		RdevKey::ShiftLeft | RdevKey::ShiftRight => SHIFT.store(pressed, Ordering::Relaxed),
		RdevKey::MetaLeft | RdevKey::MetaRight => WIN.store(pressed, Ordering::Relaxed),
		_ => {}
	}
}

fn modifiers_match(h: &Hotkey) -> bool {
	h.ctrl == CTRL.load(Ordering::Relaxed)
		&& h.alt == ALT.load(Ordering::Relaxed)
		&& h.shift == SHIFT.load(Ordering::Relaxed)
		&& h.win == WIN.load(Ordering::Relaxed)
}

/// Translates a physical rdev key into a canonical name for comparison
/// with the Hotkey.key part (the one that comes after ^!+# in the config).
/// Covers letters, numbers, F1-F12 and basic function keys.
fn key_to_name(key: RdevKey) -> Option<&'static str> {
	use RdevKey::*;
	Some(match key {
		KeyA => "a",
		KeyB => "b",
		KeyC => "c",
		KeyD => "d",
		KeyE => "e",
		KeyF => "f",
		KeyG => "g",
		KeyH => "h",
		KeyI => "i",
		KeyJ => "j",
		KeyK => "k",
		KeyL => "l",
		KeyM => "m",
		KeyN => "n",
		KeyO => "o",
		KeyP => "p",
		KeyQ => "q",
		KeyR => "r",
		KeyS => "s",
		KeyT => "t",
		KeyU => "u",
		KeyV => "v",
		KeyW => "w",
		KeyX => "x",
		KeyY => "y",
		KeyZ => "z",
		Num0 => "0",
		Num1 => "1",
		Num2 => "2",
		Num3 => "3",
		Num4 => "4",
		Num5 => "5",
		Num6 => "6",
		Num7 => "7",
		Num8 => "8",
		Num9 => "9",
		F1 => "f1",
		F2 => "f2",
		F3 => "f3",
		F4 => "f4",
		F5 => "f5",
		F6 => "f6",
		F7 => "f7",
		F8 => "f8",
		F9 => "f9",
		F10 => "f10",
		F11 => "f11",
		F12 => "f12",
		Space => "space",
		Return => "enter",
		Tab => "tab",
		Escape => "esc",
		Backspace => "backspace",
		Delete => "delete",
		Home => "home",
		End => "end",
		PageUp => "pageup",
		PageDown => "pagedown",
		UpArrow => "up",
		DownArrow => "down",
		LeftArrow => "left",
		RightArrow => "right",
		_ => return None,
	})
}

/// Looks for a binding in the config that matches the current modifiers + pressed key.
/// Returns the text that needs to be printed.
pub fn find_match(key: RdevKey, keybinds: &HashMap<String, String>) -> Option<&str> {
	let name = key_to_name(key)?;
	keybinds.iter().find_map(|(spec, text)| {
		let hotkey = parse(spec)?;
		if hotkey.key == name && modifiers_match(&hotkey) {
			Some(text.as_str())
		} else {
			None
		}
	})
}
