use eframe::egui::{self, RichText};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Default)]
pub struct State {
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
    let dir = crate::util::downloads_dir()?;
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

pub fn ui(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("Download files");
    ui.separator();
    
    let mut finished = None;
    if let Some(rx) = &state.rx {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Progress(p) => state.progress = p,
                Msg::Done(result) => finished = Some(result),
            }
        }
    }
    match finished {
        Some(Ok(())) => {
            state.progress = 1.0;
            state.status = "Download completed! (saved in ~/Downloads)".to_string();
            state.rx = None;
        }
        Some(Err(e)) => {
            state.status = e;
            state.rx = None;
        }
        None => {}
    }

    let downloading = state.rx.is_some();

    ui.label("URL");
    ui.add(
        egui::TextEdit::singleline(&mut state.url)
            .hint_text("https://example.com/file.zip")
            .desired_width(f32::INFINITY),
    );

    let can_start = !downloading && !state.url.trim().is_empty();
    let button = egui::Button::new(RichText::new("Download").size(16.0)).corner_radius(10.0);
    if ui
        .add_enabled_ui(can_start, |ui| ui.add_sized([ui.available_width(), 36.0], button))
        .inner
        .clicked()
    {
        state.progress = 0.0;
        state.status = "Downloading…".to_string();
        state.rx = Some(start_download(state.url.trim().to_string(), ui.ctx().clone()));
    }

    if downloading || state.progress > 0.0 {
        ui.add(egui::ProgressBar::new(state.progress).show_percentage());
    }
    if !state.status.is_empty() {
        ui.label(&state.status);
    }
}