use crate::app::App;
use eframe::egui;
use std::sync::atomic::Ordering;

pub fn show(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        let current = app.cursor.load(Ordering::Relaxed);
        let guard = app.images.lock().unwrap();
        let total = guard.len();
        let img = guard.get(current);

        let filename = img
            .and_then(|img| img.path.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let size_mb = img
            .and_then(|img| std::fs::metadata(&img.path).ok())
            .map(|m| format!("{:.2} MB", m.len() as f64 / 1_048_576.0))
            .unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label(filename);
            ui.separator();
            ui.label(&size_mb);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} / {}", current + 1, total));
            });
        });
    });

    if app.is_loading.load(Ordering::Relaxed) {
        ui.centered_and_justified(|ui| {
            ui.spinner();
        });
        return;
    }

    if let Some(tex) = app.current_texture().as_ref() {
        let size = tex.size_vec2();
        egui::containers::Scene::new()
            .zoom_range(0.0..=f32::INFINITY)
            .show(ui, &mut app.scene_rect, |ui| {
                ui.image(egui::load::SizedTexture::new(tex.id(), size));
            });
    }
}
