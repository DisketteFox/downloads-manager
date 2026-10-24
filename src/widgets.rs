use eframe::egui::{self, Color32, RichText};

pub fn big_button(ui: &mut egui::Ui, text: &str, fill: Option<Color32>) -> egui::Response {
    let mut label = RichText::new(text).size(17.0);
    if fill.is_some() {
        label = label.color(Color32::WHITE);
    }
    let mut button = egui::Button::new(label).corner_radius(10.0);
    if let Some(color) = fill {
        button = button.fill(color);
    }
    ui.add_sized([ui.available_width(), 44.0], button)
}

pub fn back_button(ui: &mut egui::Ui) -> bool {
    let icon = egui::Image::new(egui::include_image!("../assets/images/arrow-left-short.svg"))
        .fit_to_exact_size(egui::vec2(16.0, 16.0))
        .tint(ui.visuals().text_color());
    ui.add(egui::Button::image_and_text(icon, "Back")).clicked()
}