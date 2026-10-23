use eframe::egui::{self, Color32, RichText};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([420.0, 460.0]),
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
    Sub(&'static str),
}

const MENU: [(&str, Page); 3] = [
    ("Download files", Page::Download),
    ("Organize files", Page::Sub("Organize files")),
    ("Manage backups", Page::Sub("Manage backups")),
];

#[derive(Default)]
struct MyApp {
    page: Page,
    url: String,
    progress: f32,
    status: String,
    rx: Option<Receiver<Msg>>,
}

enum Msg {
    Progress(f32),
    Done(Result<(), String>),
}

fn start_download(url: String, ctx: egui::Context) -> Receiver<Msg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = run_wget(&url, &tx, &ctx);
        let _ = tx.send(Msg::Done(result));
        ctx.request_repaint();
    });
    rx
}

fn run_wget(url: &str, tx: &Sender<Msg>, ctx: &egui::Context) -> Result<(), String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not find your home directory".to_string())?;
    let dir = PathBuf::from(home).join("Downloads");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut child = Command::new("wget")
        .args(["--progress=dot", "-P"])
        .arg(&dir)
        .arg("--")
        .arg(url)
        .env("LC_ALL", "C")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Could not start wget (is it installed?): {e}"))?;

    let stderr = child.stderr.take().ok_or("Could not read wget output")?;
    let mut last_line = String::new();

    for line in BufReader::new(stderr).split(b'\n') {
        let line = String::from_utf8_lossy(&line.map_err(|e| e.to_string())?).into_owned();

        if let Some(percent) = line
            .split_whitespace()
            .find_map(|word| word.strip_suffix('%')?.parse::<f32>().ok())
        {
            let _ = tx.send(Msg::Progress(percent / 100.0));
            ctx.request_repaint();
        }
        if !line.trim().is_empty() {
            last_line = line.trim().to_string();
        }
    }

    if child.wait().map_err(|e| e.to_string())?.success() {
        Ok(())
    } else {
        Err(format!("Download failed: {last_line}"))
    }
}


fn big_button(ui: &mut egui::Ui, text: &str, fill: Option<Color32>) -> egui::Response {
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

impl MyApp {
    fn download_ui(&mut self, ui: &mut egui::Ui) {
        let mut finished = None;
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    Msg::Progress(p) => self.progress = p,
                    Msg::Done(result) => finished = Some(result),
                }
            }
        }
        match finished {
            Some(Ok(())) => {
                self.progress = 1.0;
                self.status = "Download completed! (saved in ~/Downloads)".to_string();
                self.rx = None;
            }
            Some(Err(e)) => {
                self.status = e;
                self.rx = None;
            }
            None => {}
        }

        let downloading = self.rx.is_some();

        ui.label("URL");
        ui.add(
            egui::TextEdit::singleline(&mut self.url)
                .hint_text("https://example.com/file.zip")
                .desired_width(f32::INFINITY),
        );

        let can_start = !downloading && !self.url.trim().is_empty();
        let button = egui::Button::new(RichText::new("Download").size(16.0)).corner_radius(10.0);
        if ui
            .add_enabled_ui(can_start, |ui| ui.add_sized([ui.available_width(), 36.0], button))
            .inner
            .clicked()
        {
            self.progress = 0.0;
            self.status = "Downloading…".to_string();
            self.rx = Some(start_download(self.url.trim().to_string(), ui.ctx().clone()));
        }

        if downloading || self.progress > 0.0 {
            ui.add(
                egui::ProgressBar::new(self.progress)
                    .show_percentage()
            );
        }
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::central_panel(ui.style()).inner_margin(24.0);

        egui::CentralPanel::default().frame(frame).show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            match self.page {
                Page::Home => {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Download Manager").size(28.0).strong());
                        ui.label(RichText::new("What would you like to do?").weak());
                    });
                    ui.add_space(12.0);
                    for (name, page) in MENU {
                        if big_button(ui, name, None).clicked() {
                            self.page = page;
                        }
                    }
                    if big_button(ui, "Codeberg", None).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("https://codeberg.org/DisketteFox"));
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
                page => {
                    let icon = egui::Image::new(egui::include_image!("../assets/arrow-left-short.svg"))
                        .fit_to_exact_size(egui::vec2(16.0, 16.0))
                        .tint(ui.visuals().text_color());
                    if ui.add(egui::Button::image_and_text(icon, "Back")).clicked() {
                        self.page = Page::Home;
                    }
                    match page {
                        Page::Download => {
                            ui.heading("Download files");
                            ui.separator();
                            self.download_ui(ui);
                        }
                        Page::Sub(name) => {
                            ui.heading(name);
                            egui::Frame::group(ui.style())
                                .corner_radius(10.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label("Content goes here.");
                                });
                        }
                        Page::Home => {}
                    }
                }
            }
        });
    }
}