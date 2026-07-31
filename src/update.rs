use serde::Deserialize;

const REPO: &str = "t1ltxz-gxd/popcorn";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Release {
	tag_name: String,
	html_url: String,
}

/// Starts checking for updates in the background so as not to block the start of the tray.
pub fn check_for_update_in_background() {
	std::thread::spawn(|| {
		if let Err(e) = check() {
			eprintln!("[update] check for updates failed: {e}");
		}
	});
}

fn check() -> anyhow::Result<()> {
	let url = format!("https://api.github.com/repos/{REPO}/releases/latest");

	let release: Release = ureq::get(&url)
		// GitHub API requires a non-empty User-Agent, otherwise it returns 403
		.header("User-Agent", "popcorn-update-checker")
		.call()?
		.body_mut()
		.read_json()?;

	let latest = release.tag_name.trim_start_matches('v');

	if is_newer(latest, CURRENT_VERSION) {
		println!(
			"[update] new version {latest} is available (you have {CURRENT_VERSION}), I open GitHub..."
		);
		if let Err(e) = open::that(&release.html_url) {
			eprintln!("[update] failed to open release page: {e}");
		}
	} else {
		println!("[update] the current version is used ({CURRENT_VERSION})");
	}

	Ok(())
}

/// Simple comparison of versions of the form "x.y.z" by numeric components,
/// without external crates like semver (enough for releases on GitHub).
fn is_newer(latest: &str, current: &str) -> bool {
	fn parse(v: &str) -> Vec<u32> {
		v.split('.').filter_map(|p| p.parse().ok()).collect()
	}
	parse(latest) > parse(current)
}
