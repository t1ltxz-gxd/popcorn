use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::collections::{HashMap, VecDeque};

const MAX_BUFFER: usize = 32;

/// A sliding buffer of the last printed characters.
pub struct InputBuffer {
	buf: VecDeque<char>,
}

impl InputBuffer {
	pub fn new() -> Self {
		Self {
			buf: VecDeque::with_capacity(MAX_BUFFER),
		}
	}

	pub fn push(&mut self, c: char) {
		self.buf.push_back(c);
		while self.buf.len() > MAX_BUFFER {
			self.buf.pop_front();
		}
	}

	pub fn clear(&mut self) {
		self.buf.clear();
	}

	pub fn as_string(&self) -> String {
		self.buf.iter().collect()
	}
}

/// Looks for the longest snippet key that ends the buffer.
/// (needed in case one key is a suffix of another)
pub fn match_snippet<'a>(
	buffer: &str,
	snippets: &'a HashMap<String, String>,
) -> Option<(&'a str, &'a str)> {
	snippets
		.iter()
		.filter(|(key, _)| !key.is_empty() && buffer.ends_with(key.as_str()))
		.max_by_key(|(key, _)| key.len())
		.map(|(k, v)| (k.as_str(), v.as_str()))
}

/// Erases the 'backspaces' of the last characters, then executes 'text_to_type'
///as a template: plain text is printed as is, and commands like
/// `{{.sleep 300}}` / `{{.shift down}}` / `{{.ctrl click}}` executed
/// as pauses and key presses.
pub fn expand(backspaces: usize, text_to_type: &str) {
	let mut enigo = match Enigo::new(&Settings::default()) {
		Ok(e) => e,
		Err(e) => {
			eprintln!("[expander] failed to initialize enigo: {e}");
			return;
		}
	};

	for _ in 0..backspaces {
		let _ = enigo.key(Key::Backspace, Direction::Click);
	}
	let _ = enigo.text(text_to_type);
}

/// Maps the key name from '{{.<name> ...}}' with type enigo::Key.
/// Not the most complete list, but covers the main modifiers and service keys.
fn key_from_name(name: &str) -> Option<Key> {
	if name.chars().count() == 1 {
		return name.chars().next().map(Key::Unicode);
	}

	Some(match name {
		"shift" => Key::Shift,
		"ctrl" | "control" => Key::Control,
		"alt" => Key::Alt,
		"win" | "meta" | "cmd" => Key::Meta,
		"enter" | "return" => Key::Return,
		"tab" => Key::Tab,
		"esc" | "escape" => Key::Escape,
		"space" => Key::Space,
		"backspace" => Key::Backspace,
		"delete" | "del" => Key::Delete,
		"home" => Key::Home,
		"end" => Key::End,
		"pageup" => Key::PageUp,
		"pagedown" => Key::PageDown,
		"up" => Key::UpArrow,
		"down" => Key::DownArrow,
		"left" => Key::LeftArrow,
		"right" => Key::RightArrow,
		_ => return None,
	})
}
