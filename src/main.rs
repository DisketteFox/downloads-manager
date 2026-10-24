mod pages;
mod util;
mod widgets;

use eframe::egui;
use pages::{backups, download, home, organize};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 460.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/images/icon.png"))
                    .expect("invalid icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Download Manager",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<MyApp>::default())
        }),
    )
}

#[derive(Default, Clone, Copy)]
enum Page {
    #[default]
    Home,
    Download,
    Organize,
    Backups,
}

#[derive(Default)]
struct MyApp {
    page: Page,
    download: download::State,
    organize: organize::State,
    backups: backups::State,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(24.0);

        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            match self.page {
                Page::Home => home::ui(ui, &mut self.page),
                page => {
                    if widgets::back_button(ui) {
                        self.page = Page::Home;
                    }
                    match page {
                        Page::Download => download::ui(ui, &mut self.download),
                        Page::Organize => organize::ui(ui, &mut self.organize),
                        Page::Backups => backups::ui(ui, &mut self.backups),
                        Page::Home => {}
                    }
                }
            }
        });
    }
}