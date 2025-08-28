use eframe::egui;

/// Render the top header panel.
pub fn show(ctx: &egui::Context) {
    let scale = ctx.pixels_per_point();
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
        ui.add_space(8.0 * scale);
        ui.horizontal(|ui| {
            ui.heading(format!("📦 App Uninstaller v{}", env!("CARGO_PKG_VERSION")));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("annguyen.apps@gmail.com");
            });
        });
        ui.add_space(6.0 * scale);
    });
}
