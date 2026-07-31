mod config;
mod expander;
mod focus;
mod hook;
mod hotkeys;
mod layout;
mod state;
mod tray;
mod update;

use std::path::PathBuf;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
	let config_path = default_config_path();
	config::ensure_default_config(&config_path)?;
	let cfg = config::load_config(&config_path)?;

	update::check_for_update_in_background();

	let state = Arc::new(state::AppState::new(cfg, config_path));

	// Setup Ctrl+C handler for graceful shutdown
	let state_clone = state.clone();
	ctrlc::set_handler(move || {
		println!("[main] Received Ctrl+C, shutting down gracefully...");
		state_clone.set_exit();
		std::process::exit(0);
	})
	.expect("Error setting Ctrl-C handler");

	// monitor config changes in the background
	config::watch_config(state.clone())?;
	// install a global keyboard hook on a separate thread
	hook::run_hook_thread(state.clone());

	// tray + menus spin on the main thread (need event loop for Windows)
	tray::run_tray(state)?;

	Ok(())
}

fn default_config_path() -> PathBuf {
	let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
	home.join(".config").join("popcorn.toml")
}
