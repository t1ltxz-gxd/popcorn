use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Global,
    Whitelist,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    pub mode: Mode,
    #[serde(default)]
    pub apps: Vec<String>,
    /// If true, the snippet will work even if the abbreviation was
    /// printed in English with the same physical keys
    /// (e.g. "/z" is recognized as ".я").
    #[serde(default)]
    pub translate_layout: bool,

}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
    pub snippets: HashMap<String, String>,
    #[serde(default)]
    pub keybinds: HashMap<String, String>,

}

/// Reads and parses toml config from disk.
pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Could not read config: {path:?}"))?;
    let config: Config =
        toml::from_str(&content).with_context(|| "Could not parse the TOML config")?;
    Ok(config)
}

/// Creates a default config if the file does not already exist.
pub fn ensure_default_config(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let default = r#"[general]
# "global" — Snippets work everywhere
# "whitelist" — Only in apps in the Apps list
mode = "global"
apps = ["Discord.exe", "Telegram.exe"]
# recognize an abbreviation, even if it is printed in English
# layout with the same physical keys (".я" == "/z")
translate_layout = true

[snippets]
".hi" = "Hello, world!"

[keybinds]
"Ctrl+Alt+F3" = "Text typed by Ctrl+Alt+F3"
"Ctrl+Shift+Space" = "Text typed by Ctrl+Shift+Space"
    "#;
    std::fs::write(path, default)?;
    Ok(())
}

/// Decides whether the expander should work in the currently active application.
pub fn should_expand(config: &Config, current_process: Option<&str>) -> bool {
    match config.general.mode {
        Mode::Global => true,
        Mode::Whitelist => match current_process {
            Some(proc) => config
                .general
                .apps
                .iter()
                .any(|p| p.eq_ignore_ascii_case(proc)),
            None => false,
        },
    }
}

/// Monitors changes in the toml file and rereads the config in `AppState`.
pub fn watch_config(state: Arc<crate::state::AppState>) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    watcher.watch(&state.config_path, RecursiveMode::NonRecursive)?;

    std::thread::spawn(move || {
        // the watcher must live while this stream is spinning
        let _watcher = watcher;
        for res in rx {
            if state.should_exit() {
                break;
            }
            if res.is_ok() {
                match state.reload_config() {
                    Ok(()) => println!("[config] re-read after change"),
                    Err(e) => eprintln!("[config] reread error: {e}"),
                }
            }
        }
    });

    Ok(())
}
