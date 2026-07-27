use active_win_pos_rs::get_active_window;

/// Returns the exe name of the active window, for example "Discord.exe".
pub fn get_foreground_process_name() -> Option<String> {
    let window = get_active_window().ok()?;
    std::path::Path::new(&window.process_path)
        .file_name()?
        .to_str()
        .map(|s| s.to_string())
}
