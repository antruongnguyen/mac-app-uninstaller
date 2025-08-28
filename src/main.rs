mod types;
mod core;
mod style;
mod osx;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // On macOS, proactively set the Dock icon from our bundle/dev resources
    #[cfg(target_os = "macos")]
    {
        osx::try_set_dock_icon_from_icns();
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "App Uninstaller",
        native_options,
        Box::new(|_cc| Ok(Box::new(ui::MacUninstallerApp::default()))),
    )
}