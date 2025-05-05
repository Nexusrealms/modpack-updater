use config::{delete_by_config, load_config, write_config};
//#![windows_subsystem = "windows"]
use eframe::egui;
use egui::{IconData, ThemePreference};
use ftp::run_over_sftp;
use generate::generate;
use local::run_local;
use mrpack::update_from_mrpack_to_local;
use std::{env, fmt::Display, path::PathBuf};
mod config;
mod ftp;
mod generate;
mod local;
mod mrpack;
const _UPDATE_ENDPOINT: &str = "/update";
fn main() {
    dotenvy::dotenv().unwrap();
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
#[derive(Clone)]
pub struct FtpLocation {
    address: String,
    port: u32,
    name: String,
    password: String,
}
struct NMUClient {
    work_folder: Option<PathBuf>,
    pack_source: PackSource,
    pack_endpoint: String,
    last_run_result: String,
    ftp_location: FtpLocation,
}
impl Default for NMUClient {
    fn default() -> Self {
        Self {
            work_folder: None,
            pack_source: PackSource::None,
            pack_endpoint: String::from(""),
            last_run_result: String::from("Not ran yet"),
            ftp_location: FtpLocation {
                address: String::from(std::env::var("DEFAULT_ADDRESS").unwrap()),
                port: std::env::var("DEFAULT_PORT").unwrap().parse().unwrap(),
                name: std::env::var("DEFAULT_NAME").unwrap(),
                password: std::env::var("DEFAULT_PASSWORD").unwrap(),
            },
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
            if ui.button("Generate").clicked() {
                self.last_run_result = match generate(self) {
                    Ok(_) => String::from("Generated!"),
                    Err(s) => String::from(s),
                }
            }
            ui.separator();
            ui.group(|ui| {
                let address_label = ui.label("Address: ");
                ui.text_edit_singleline(&mut self.ftp_location.address)
                    .labelled_by(address_label.id);
                let name_label = ui.label("Name: ");
                ui.text_edit_singleline(&mut self.ftp_location.name)
                    .labelled_by(name_label.id);
                let password_label = ui.label("Password: ");
                ui.text_edit_singleline(&mut self.ftp_location.password)
                    .labelled_by(password_label.id);
                let port_label = ui.label("Port: ");
                ui.add(egui::DragValue::new(&mut self.ftp_location.port).speed(10))
                    .labelled_by(port_label.id);
            });
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
        return run_local(folder, &nmu.pack_source);
    } else if !nmu.ftp_location.address.is_empty() {
        return run_over_sftp(nmu.ftp_location.clone(), nmu.pack_source.clone());
    }
    Err("No work location set!")
}
#[derive(Clone)]
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
