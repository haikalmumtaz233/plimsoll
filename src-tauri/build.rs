const COMMANDS: &[&str] = &[
    "usage_summary",
    "set_accurate_mode",
    "set_preferences",
    "set_manual_percent",
    "set_autostart",
];

fn main() {
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS));
    if let Err(error) = tauri_build::try_build(attributes) {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
