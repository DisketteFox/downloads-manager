use crate::util::downloads_dir;
use crate::widgets::big_button;
use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(Default)]
pub struct State {
    status: String,
}

pub fn ui(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("Manage your backups");
    ui.separator();

    if big_button(ui, "Create backup", None).clicked() {
        if let Some(files) = pick_files(false) {
            state.status = create_backups(&files);
        }
    }
    if big_button(ui, "Restore backup", None).clicked() {
        if let Some(files) = pick_files(true) {
            state.status = restore_backups(&files);
        }
    }

    if !state.status.is_empty() {
        ui.label(&state.status);
    }
}

fn pick_files(only_bak: bool) -> Option<Vec<PathBuf>> {
    let mut dialog = rfd::FileDialog::new();
    if let Ok(dir) = downloads_dir() {
        dialog = dialog.set_directory(dir);
    }
    if only_bak {
        dialog = dialog.add_filter("Backup files", &["bak"]);
    }
    dialog.pick_files()
}

fn create_backups(files: &[PathBuf]) -> String {
    let mut done = 0;
    let mut errors = Vec::new();

    for file in files {
        let mut backup = file.as_os_str().to_owned();
        backup.push(".bak");

        match fs::copy(file, PathBuf::from(backup)) {
            Ok(_) => done += 1,
            Err(e) => errors.push(format!("{}: {e}", file.display())),
        }
    }
    summary("created", done, errors)
}

fn restore_backups(files: &[PathBuf]) -> String {
    let mut done = 0;
    let mut errors = Vec::new();

    for file in files {
        if file.extension().and_then(|e| e.to_str()) != Some("bak") {
            errors.push(format!("{}: not a .bak file", file.display()));
            continue;
        }
        let original = file.with_extension(""); // drops only the ".bak"

        match fs::copy(file, &original) {
            Ok(_) => done += 1,
            Err(e) => errors.push(format!("{}: {e}", file.display())),
        }
    }
    summary("restored", done, errors)
}

fn summary(verb: &str, done: usize, errors: Vec<String>) -> String {
    let mut text = format!("{done} backup(s) successfully {verb}");
    if !errors.is_empty() {
        text += &format!("\nFailed: {}", errors.join("; "));
    }
    text
}