use eframe::egui;

/// Render the top header panel.
pub fn show(ctx: &egui::Context) {
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new(format!("🗑 APP UNINSTALLER v{}", env!("CARGO_PKG_VERSION")))
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("annguyen.apps@gmail.com");
            });
        });
        ui.add_space(6.0);
    });
}
