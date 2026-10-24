use crate::util::downloads_dir;
use crate::widgets::big_button;
use eframe::egui;
use std::fs;

#[derive(Default, Clone, Copy, PartialEq)]
enum Mode {
    #[default]
    Extension,
    Date,
    Name,
}

impl Mode {
    const ALL: [Mode; 3] = [Mode::Extension, Mode::Date, Mode::Name];

    fn label(self) -> &'static str {
        match self {
            Mode::Extension => "By file extension",
            Mode::Date => "By date",
            Mode::Name => "By name",
        }
    }
}

#[derive(Default)]
pub struct State {
    mode: Mode,
    status: String,
}

pub fn ui(ui: &mut egui::Ui, state: &mut State) {
    ui.heading("Organize files");
    ui.separator();

    egui::ComboBox::from_id_salt("organize_mode")
        .selected_text(state.mode.label())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for mode in Mode::ALL {
                ui.selectable_value(&mut state.mode, mode, mode.label());
            }
        });

    if big_button(ui, "Order", None).clicked() {
        state.status = match organize(state.mode) {
            Ok((moved, 0)) => format!("Files successfully organized ({moved} moved)"),
            Ok((moved, skipped)) => format!("{moved} moved, {skipped} skipped (name already taken)"),
            Err(e) => e,
        };
    }
    if big_button(ui, "Remove duplicates", None).clicked() {
        state.status = match remove_duplicates() {
            Ok(removed) => format!("{removed} duplicate(s) removed"),
            Err(e) => e,
        };
    }

    if !state.status.is_empty() {
        ui.label(&state.status);
    }
}

fn organize(mode: Mode) -> Result<(usize, usize), String> {
    let dir = downloads_dir()?;
    let (mut moved, mut skipped) = (0, 0);

    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if !path.is_file() || name.starts_with('.') || name == "desktop.ini" {
            continue;
        }

        let folder = match mode {
            Mode::Extension => path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "no extension".to_string()),
            Mode::Name => match name.chars().next() {
                Some(c) => c.to_uppercase().to_string(),
                None => continue,
            },
            Mode::Date => {
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                let time = meta
                    .created()
                    .or_else(|_| meta.modified())
                    .map_err(|e| e.to_string())?;
                let date: chrono::DateTime<chrono::Local> = time.into();
                date.format("%Y-%m-%d").to_string()
            }
        };

        let folder_path = dir.join(&folder);
        let dest = folder_path.join(&name);

        if fs::create_dir_all(&folder_path).is_err() || dest.exists() {
            skipped += 1;
            continue;
        }
        fs::rename(&path, &dest).map_err(|e| format!("Could not move {name}: {e}"))?;
        moved += 1;
    }
    Ok((moved, skipped))
}

fn remove_duplicates() -> Result<usize, String> {
    let dir = downloads_dir()?;
    let mut removed = 0;

    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if !path.is_file() {
            continue;
        }
        if let Some((base, suffix)) = name.rsplit_once('.') {
            let is_number = !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit());
            if is_number && dir.join(base).exists() {
                fs::remove_file(&path).map_err(|e| format!("Could not delete {name}: {e}"))?;
                removed += 1;
            }
        }
    }
    Ok(removed)
}