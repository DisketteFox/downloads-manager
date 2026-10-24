use crate::widgets::big_button;
use crate::Page;
use eframe::egui::{self, Color32, RichText};

const MENU: [(&str, Page); 3] = [
    ("Download files", Page::Download),
    ("Organize files", Page::Organize),
    ("Manage backups", Page::Backups),
];

pub fn ui(ui: &mut egui::Ui, current_page: &mut Page) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Download Manager").size(28.0).strong());
        ui.label(RichText::new("by DisketteFox").weak());
    });
    ui.add_space(12.0);

    for (name, page) in MENU {
        if big_button(ui, name, None).clicked() {
            *current_page = page;
        }
    }
    if big_button(ui, "Exit", None).clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    ui.add_space(8.0);
    ui.separator();
    ui.label(RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).weak());
    ui.horizontal(|ui| {
        if ui.link("Codeberg").clicked() {
            ui.ctx()
                .open_url(egui::OpenUrl::new_tab("https://codeberg.org/DisketteFox"));
        }
        ui.label(RichText::new("·").size(20.0));
        if ui.link("Github").clicked() {
            ui.ctx()
                .open_url(egui::OpenUrl::new_tab("https://github.com/DisketteFox"));
        }
    });
}