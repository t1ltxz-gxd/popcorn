use std::sync::Arc;
use std::time::{Duration, Instant};

use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

use crate::state::AppState;

pub fn run_tray(state: Arc<AppState>) -> anyhow::Result<()> {
	let event_loop = EventLoop::new();

	let quit_item = MenuItem::new("Exit", true, None);
	let pause_item = MenuItem::new("Pause", true, None);
	let settings_item = MenuItem::new("Edit config", true, None);

	let quit_id = quit_item.id().clone();
	let pause_id = pause_item.id().clone();
	let settings_id = settings_item.id().clone();

	let menu = Menu::new();
	menu.append_items(&[&settings_item, &pause_item, &quit_item])?;

	// Icons are built into the binary itself at the compilation stage - it works
	// the same in debug and release, there is no need to copy assets next to the exe.
	let normal_icon = load_icon(include_bytes!("../assets/favicon.png"));
	let pause_icon = load_icon(include_bytes!("../assets/pause.png"));

	let tray = TrayIconBuilder::new()
		.with_menu(Box::new(menu))
		.with_icon(normal_icon.clone())
		.with_tooltip("Popcorn")
		.build()?;

	let menu_channel = MenuEvent::receiver();

	event_loop.run(move |_event, _target, control_flow| {
		// don't load the CPU with a busy loop, but don't sleep for too long -
		// otherwise the menu will respond to clicks with a delay
		*control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));

		if let Ok(event) = menu_channel.try_recv() {
			if event.id == quit_id {
				println!("[tray] Quitting from menu...");
				state.set_exit();
				std::process::exit(0);
			} else if event.id == pause_id {
				let now_paused = state.toggle_pause();
				pause_item.set_text(if now_paused { "Continue" } else { "Pause" });

				let icon = if now_paused {
					pause_icon.clone()
				} else {
					normal_icon.clone()
				};
				if let Err(e) = tray.set_icon(Some(icon)) {
					eprintln!("[tray] failed to change icon: {e}");
				}
			} else if event.id == settings_id
				&& let Err(e) = open::that(&state.config_path)
			{
				eprintln!("[tray] failed to open config: {e}");
			}
		}
	});
}

/// Tries to load `assets/<file_name>` next to the exe (in debug - from the root
/// project, see `icon_path`). If there is no file, or it is broken, it is used
/// generated stub (blue square) so that the application does not crash
/// due to a missing picture.
/// Decodes the icon from the bytes built into the binary via `include_bytes!`.
/// If the PNG is broken for some reason, the generated stub is used
/// (blue square) to prevent the application from crashing.
fn load_icon(png_bytes: &[u8]) -> Icon {
	match image::load_from_memory(png_bytes) {
		Ok(img) => {
			let img = img.into_rgba8();
			let (width, height) = img.dimensions();
			Icon::from_rgba(img.into_raw(), width, height).expect("failed to create tray icon")
		}
		Err(e) => {
			eprintln!("[tray] failed to decode embedded icon ({e}), using a stub");
			fallback_icon()
		}
	}
}

fn fallback_icon() -> Icon {
	const SIZE: u32 = 32;
	let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
	for _ in 0..(SIZE * SIZE) {
		rgba.extend_from_slice(&[0x4A, 0x9E, 0xE0, 0xFF]);
	}
	Icon::from_rgba(rgba, SIZE, SIZE).expect("failed to create tray icon")
}
