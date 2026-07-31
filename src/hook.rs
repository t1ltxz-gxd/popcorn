use std::sync::{Arc, Mutex, OnceLock};

use rdev::{Event, EventType, Key as RdevKey, listen};

use crate::config::should_expand;
use crate::expander::{InputBuffer, expand, match_snippet};
use crate::focus::get_foreground_process_name;
use crate::hotkeys;
use crate::layout::translate_en_to_ru;
use crate::state::AppState;

// The rdev callback does not know how to capture the environment by value as conveniently as
// like a closure with move in a separate thread, so the state is static.
static STATE: OnceLock<Arc<AppState>> = OnceLock::new();
static BUFFER: OnceLock<Mutex<InputBuffer>> = OnceLock::new();

/// Runs the global keyboard listener on a separate thread.
/// `rdev::listen` blocks the thread, so we run it on a separate one.
pub fn run_hook_thread(state: Arc<AppState>) {
	STATE.set(state).ok();
	BUFFER.set(Mutex::new(InputBuffer::new())).ok();

	std::thread::spawn(|| {
		if let Err(e) = listen(handle_event) {
			eprintln!("[hook] не удалось запустить глобальный листенер: {e:?}");
		}
	});
}

fn handle_event(event: Event) {
	match event.event_type {
		EventType::KeyPress(key) => {
			hotkeys::update_modifier_state(key, true);

			// if the key combination worked, we don’t count it
			// symbol for the buffer of regular snippets
			if try_trigger_hotkey(key) {
				return;
			}

			// event.name is a symbol already translated for the current layout,
			// comes only to KeyPress for keys that print text
			if let Some(text) = event.name {
				for ch in text.chars() {
					handle_char(ch);
				}
			}
		}
		EventType::KeyRelease(key) => {
			hotkeys::update_modifier_state(key, false);
		}
		_ => {}
	}
}

/// Checks if the current combination of modifiers + key
/// matches one of the bindings in the config, and if so, prints the associated text.
fn try_trigger_hotkey(key: RdevKey) -> bool {
	let Some(state) = STATE.get() else {
		return false;
	};

	if state.is_paused() {
		return false;
	}

	let config = state.config.read().unwrap();
	let current_process = get_foreground_process_name();
	if !should_expand(&config, current_process.as_deref()) {
		return false;
	}

	match hotkeys::find_match(key, &config.keybinds) {
		Some(text) => {
			let text = text.to_string();
			println!("[hook] сработала комбинация клавиш (процесс: {current_process:?})");
			drop(config);
			expand(0, &text);
			true
		}
		None => false,
	}
}

fn handle_char(ch: char) {
	let Some(state) = STATE.get() else { return };
	let Some(buffer_lock) = BUFFER.get() else {
		return;
	};

	if state.is_paused() {
		return;
	}

	let config = state.config.read().unwrap();
	let current_process = get_foreground_process_name();
	if !should_expand(&config, current_process.as_deref()) {
		return;
	}

	let mut buffer = buffer_lock.lock().unwrap();
	buffer.push(ch);

	// We try to match the snippet only by the trigger — pressing space.
	// Any other characters just accumulate in the buffer.
	if ch != ' ' {
		return;
	}

	let text = buffer.as_string();
	// text without the just entered space - we look for the prefix using it
	let prefix = &text[..text.len() - ch.len_utf8()];

	// first we try as is, and if we don’t find it and the option is enabled -
	// try the same string, “translated” from the EN layout to RU
	// (for example, "/z" is recognized as ".я")
	let translated;
	let matched = match match_snippet(prefix, &config.snippets) {
		Some(found) => Some(found),
		None if config.general.translate_layout => {
			translated = translate_en_to_ru(prefix);
			match_snippet(&translated, &config.snippets)
		}
		None => None,
	};

	if let Some((key, replacement)) = matched {
		println!("[hook] сработал снипет {key:?} (процесс: {current_process:?})");
		// erase the key itself + space trigger, then print the replacement and return the space
		let backspaces = key.chars().count() + 1;
		let text_to_type = if replacement.ends_with(char::is_whitespace) {
			replacement.to_string()
		} else {
			format!("{replacement} ")
		};
		buffer.clear();
		drop(buffer);
		drop(config);
		expand(backspaces, &text_to_type);
	}
}
