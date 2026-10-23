use eframe::egui::{self, Color32, RichText};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 460.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Download Manager",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default, Clone, Copy)]
enum Page {
    #[default]
    Home,
    Sub(&'static str),
}

#[derive(Default)]
struct MyApp {
    page: Page,
}

const MENU: [&str; 3] = ["Download files", "Organize files", "Manage backups"];

fn big_button(ui: &mut egui::Ui, text: &str, fill: Option<Color32>) -> egui::Response {
    let mut label = RichText::new(text).size(17.0);
    let mut button_fill = None;
    if let Some(color) = fill {
        label = label.color(Color32::WHITE);
        button_fill = Some(color);
    }
    let mut button = egui::Button::new(label).corner_radius(10.0);
    if let Some(color) = button_fill {
        button = button.fill(color);
    }
    ui.add_sized([ui.available_width(), 44.0], button)
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(24.0);

        egui::CentralPanel::default().frame(frame).show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Download Manager").size(28.0).strong());
                ui.label(RichText::new("by DisketteFox").weak());
            });
            ui.add_space(12.0);

            match self.page {
                Page::Home => {
                    for name in MENU {
                        if big_button(ui, name, None).clicked() {
                            self.page = Page::Sub(name);
                        }
                    }
                    if big_button(ui, "Codeberg", None).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("https://codeberg.org"));
                    }
                    if big_button(ui, "Exit", Some(Color32::from_rgb(190, 60, 60))).clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Theme");
                        egui::widgets::global_theme_preference_buttons(ui);
                    });
                }
                Page::Sub(name) => {
                    if ui.button("Back").clicked() {
                        self.page = Page::Home;
                    }
                    ui.heading(name);

                    egui::Frame::group(ui.style())
                        .corner_radius(10.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label("Content goes here.");
                        });
                }
            }
        });
    }
}