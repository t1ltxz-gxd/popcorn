use std::path::PathBuf;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::Config;

pub struct AppState {
	paused: AtomicBool,
	pub should_exit: AtomicBool,
	pub config: RwLock<Config>,
	pub config_path: PathBuf,
}

impl AppState {
	pub fn new(config: Config, config_path: PathBuf) -> Self {
		Self {
			paused: AtomicBool::new(false),
			should_exit: AtomicBool::new(false),
			config: RwLock::new(config),
			config_path,
		}
	}

	pub fn is_paused(&self) -> bool {
		self.paused.load(Ordering::Relaxed)
	}

	/// Toggles pause, returns new value.
	pub fn toggle_pause(&self) -> bool {
		let new_val = !self.is_paused();
		self.paused.store(new_val, Ordering::Relaxed);
		new_val
	}

	pub fn should_exit(&self) -> bool {
		self.should_exit.load(Ordering::Relaxed)
	}

	pub fn set_exit(&self) {
		self.should_exit.store(true, Ordering::Relaxed);
	}

	pub fn reload_config(&self) -> anyhow::Result<()> {
		let fresh = crate::config::load_config(&self.config_path)?;
		*self.config.write().unwrap() = fresh;
		Ok(())
	}
}
