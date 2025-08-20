use constcat::concat;
use eframe::egui;
use std::{
    io::{BufRead as _, BufReader},
    path::{Path, PathBuf},
    thread,
};

use crate::injector::install_mod;

pub struct SnaxAPLoader {
    exe_location: Option<PathBuf>,
    exe_location_user_str: String,
    has_launched: bool,
}

impl Default for SnaxAPLoader {
    fn default() -> Self {
        let snax_path = detect_bugsnax_location();
        let path_string = snax_path.clone().map_or("".to_string(), |p| {
            p.into_os_string()
                .into_string()
                .expect("bugsnax.exe path not representable as string!")
        });
        Self {
            exe_location: snax_path,
            exe_location_user_str: path_string,
            has_launched: false,
        }
    }
}

impl eframe::App for SnaxAPLoader {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Snax Archipelago Launcher");
            if self.has_launched {
                self.log_watch(ui);
            } else {
                self.exe_selector(ui);
                ctx.request_repaint(); //update window even when out of focus
            };
        });
    }
}

impl SnaxAPLoader {
    fn exe_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let name_label = ui.label("Bugsnax location:");
            ui.add_enabled_ui(false, |ui| {
                ui.text_edit_singleline(&mut self.exe_location_user_str)
                    .labelled_by(name_label.id);
            });
            if ui.button("Select Bugsnax.exe location").clicked() {
                self.exe_location = rfd::FileDialog::new().pick_file();
                self.exe_location_user_str = self.exe_location.clone().map_or("".to_string(), |p| {
                    p.to_str()
                        .expect("could not convert user-provided bugsnax path to string")
                        .to_string()
                })
            }
        });
        if ui.button("Launch Bugsnax Archipelago!").clicked() {
            let stdout = install_mod(
                self.exe_location
                    .as_ref()
                    .expect("Can't launch bugsnax without knowing where it is!"),
            );
            thread::spawn(move || {
                let reader = BufReader::new(stdout);

                for line in reader.lines() {
                    let line = line.expect("could not read line from snax stdout");
                    log::info!("{line}");
                }
            });
            self.has_launched = true;
        }
    }

    fn log_watch(&mut self, ui: &mut egui::Ui) {
        egui_logger::logger_ui().show(ui);
    }
}

const BUGSNAX_DIR: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Bugsnax"#;

const BUGSNAX_EXE_PATH: &str = concat!(BUGSNAX_DIR, "\\Bugsnax.exe");

fn detect_bugsnax_location() -> Option<PathBuf> {
    let path = Path::new(BUGSNAX_EXE_PATH).to_path_buf();
    if path.exists() { Some(path) } else { None }
}
