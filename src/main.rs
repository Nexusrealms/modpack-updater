use config::{delete_by_config, load_config, write_config};
//#![windows_subsystem = "windows"]
use eframe::egui;
use egui::{IconData, ThemePreference};
use generate::generate;
use mrpack::update_from_mrpack;
use std::{fmt::Display, path::PathBuf};
mod config;
mod generate;
mod mrpack;
const _UPDATE_ENDPOINT: &str = "/update";
fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "NMU",
        options,
        Box::new(|_cc| Ok(Box::<NMUClient>::default())),
    )
    .expect("Did not gui");
}
fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../icon.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
struct NMUClient {
    work_folder: Option<PathBuf>,
    pack_source: PackSource,
    pack_endpoint: String,
    last_run_result: String,
}
impl Default for NMUClient {
    fn default() -> Self {
        Self {
            work_folder: None,
            pack_source: PackSource::None,
            pack_endpoint: String::from(""),
            last_run_result: String::from("Not ran yet"),
        }
    }
}
impl eframe::App for NMUClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(ThemePreference::Dark);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Nexusrealms modpack updater");
            if ui.button("Select work folder").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.work_folder = Some(path);
                    if let Ok(updater_config) =
                        load_config(&self.work_folder.as_ref().unwrap().as_path())
                    {
                        if let Some(url) = updater_config.pack_endpoint {
                            self.pack_endpoint = url.clone();
                            self.pack_source = PackSource::Url(url);
                        }
                    }
                }
            }
            if let Some(path) = &self.work_folder {
                ui.label("Work folder: ");
                ui.monospace(format!("{}", path.display()));
            }
            ui.end_row();
            ui.horizontal(|ui| {
                if ui.button("Select pack source").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Modrinth Modpack File", &["mrpack"])
                        .pick_file()
                    {
                        self.pack_source = PackSource::FromFile(path);
                    }
                }
                let label = ui.label("Pack endpoint: ");
                ui.text_edit_singleline(&mut self.pack_endpoint)
                    .labelled_by(label.id);
                if ui.button("Set").clicked() {
                    self.pack_source = PackSource::Url(self.pack_endpoint.clone())
                }
            });
            ui.label("Pack source: ");
            ui.monospace(format!("{}", &self.pack_source));
            ui.end_row();
            if ui.button("Run").clicked() {
                self.last_run_result = match run(self) {
                    Ok(_) => String::from("Ran!"),
                    Err(s) => String::from(s),
                }
            }
            if ui.button("Update").clicked() {
                self.last_run_result = match update(self) {
                    Ok(_) => String::from("Updated!"),
                    Err(s) => String::from(s),
                }
            }
            if ui.button("Generate").clicked() {
                self.last_run_result = match generate(self) {
                    Ok(_) => String::from("Generated!"),
                    Err(s) => String::from(s),
                }
            }
            ui.label(&self.last_run_result);
        });
    }
}
fn update(_nmu: &NMUClient) -> Result<(), &'static str> {
    Err("Update checking is not implemented")
    /*if let PackSource::Url(url) = &nmu.pack_source {
        if let Ok(response) = reqwest::blocking::get(url.clone() + UPDATE_ENDPOINT) {
            if let Ok(boolean) = serde_json::from_str(
                response
                    .text()
                    .expect("Api response was not readable as text and im tired of if lets")
                    .as_str(),
            ) {
                if boolean {
                    run(nmu)
                } else {
                    Err("Pack is not updated")
                }
            } else {
                Err("Could not deserialize update check response")
            }
        } else {
            Err("Could not GET from update endpoint")
        }
    } else {
        Err("Pack source does not support update checking")
    }*/
}
fn run(nmu: &NMUClient) -> Result<(), &'static str> {
    if let Some(folder) = &nmu.work_folder {
        return match nmu.pack_source {
            PackSource::None => Err("No pack source set!"),
            _ => {
                let folder_path = folder.as_path();
                if let Ok(config) = load_config(folder_path) {
                    match delete_by_config(folder_path, &config) {
                        Ok(_) => {}
                        Err(err) => {
                            return Err(err);
                        }
                    };
                };
                match update_from_mrpack(&nmu.pack_source, folder) {
                    Ok(config) => write_config(&folder, &config),
                    Err(str) => Err(str),
                }
            }
        };
    }
    Err("No work folder set!")
}
enum PackSource {
    FromFile(PathBuf),
    Url(String),
    None,
}
impl Display for PackSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackSource::FromFile(path) => write!(f, "{}", path.display()),
            PackSource::Url(string) => write!(f, "{}", string),
            PackSource::None => write!(f, "None selected"),
        }
    }
}
