use eframe::egui::{self, Layout};
use std::path::PathBuf;

pub enum Action {
    OpenImage(PathBuf),
    OpenFolder(PathBuf),
}

pub fn show(ui: &mut egui::Ui) -> Option<Action> {
    let mut action = None;
    ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(ui.available_height() / 2.0 - 60.0);
        ui.heading("Lyt");
        ui.add_space(24.0);
        let btn = egui::vec2(160.0, 44.0);
        if ui
            .add_sized(btn, egui::Button::new("📂  Open Image"))
            .clicked()
        {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Images", &["jpg", "jpeg"])
                .pick_file()
            {
                action = Some(Action::OpenImage(p));
            }
        }
        ui.add_space(12.0);
        if ui
            .add_sized(btn, egui::Button::new("🗂  Open Folder"))
            .clicked()
        {
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                action = Some(Action::OpenFolder(p));
            }
        }
        ui.add_space(16.0);
        ui.label(
            egui::RichText::new("press [ O ] -> image   press [ Ctrl + O ] -> folder")
                .small()
                .weak(),
        );
    });
    action
}
