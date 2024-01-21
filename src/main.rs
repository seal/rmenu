#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use freedesktop_entry_parser::parse_entry;
use std::env;

use std::{fs, process::Command};

use eframe::egui::{self, Key};
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_fullscreen(true),
        ..Default::default()
    };
    eframe::run_native(
        "RMenu",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    applications: Vec<Application>,
    search: String,
}
#[derive(Clone, Debug)]
struct Application {
    path: Option<String>,
    name: String,
    exec_file: String,
    icon_path: String,
}
fn get_files(path: &str) -> Vec<Application> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.contains(".desktop") {
                        //files.push(file_name.to_owned());
                        let path = format!("/usr/share/applications/{}", file_name);
                        let entry = parse_entry(path.clone()).unwrap();
                        let name = entry
                            .section("Desktop Entry")
                            .attr("Name")
                            .map(|s| s.to_string());
                        let mut exec_file = entry
                            .section("Desktop Entry")
                            .attr("TryExec")
                            .map(|s| s.to_string());
                        if exec_file.is_none() {
                            exec_file = entry
                                .section("Desktop Entry")
                                .attr("Exec")
                                .map(|s| s.to_string());
                        }
                        let icon_path = entry
                            .section("Desktop Entry")
                            .attr("Icon")
                            .map(|s| s.to_string());
                        if icon_path.is_some() && exec_file.is_some() && name.is_some() {
                            files.push(Application {
                                path: Some(path),
                                name: name.unwrap(),
                                exec_file: exec_file.unwrap(),
                                icon_path: icon_path.unwrap(),
                            })
                        }
                    }
                }
            }
        }
    }
    files
}
fn search_strings(apps: &Vec<Application>, search_term: &str) -> Vec<Application> {
    let mut result = Vec::with_capacity(apps.len());

    for s in apps {
        if s.name.clone().to_lowercase().contains(search_term) {
            result.push(s.clone());
        }
    }

    result
}
impl Default for MyApp {
    fn default() -> Self {
        /*
         * usr/share/icons/hicolor/
         */
        Self {
            applications: get_files("/usr/share/applications/"),
            search: "".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                std::process::exit(1);
            }
            ui.heading("Rmenu");
            ui.text_edit_singleline(&mut self.search);
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.search == "" {
                    for app in self.applications.iter() {
                        if ui.button(app.name.clone()).clicked() {
                            open_application(app);
                        }
                    }
                } else {
                    let applications = search_strings(&self.applications, &self.search.as_str());
                    for app in applications.iter() {
                        if ui.button(app.name.clone()).clicked() {
                            open_application(app);
                        }
                    }
                }
            });
        });
    }
}

fn open_application(app: &Application) {
    if let Some(terminal_command) = get_terminal_command() {
        let desktop_file_path = app.path.clone().unwrap_or_else(|| "".to_string());
        let application_name = std::path::Path::new(&desktop_file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        match Command::new("gtk-launch").arg(application_name).spawn() {
            Ok(_) => {}
            Err(e) => eprintln!("Error launching application: {}", e),
        }
    } else {
        eprintln!("No terminal command found. Cannot launch application.");
    }
}

fn get_terminal_command() -> Option<String> {
    let terminals = vec![
        env::var("TERMINAL").ok(),
        Some("x-terminal-emulator".to_string()),
        Some("mate-terminal".to_string()),
        Some("gnome-terminal".to_string()),
        Some("terminator".to_string()),
        Some("xfce4-terminal".to_string()),
        Some("urxvt".to_string()),
        Some("rxvt".to_string()),
        Some("termit".to_string()),
        Some("Eterm".to_string()),
        Some("aterm".to_string()),
        Some("uxterm".to_string()),
        Some("xterm".to_string()),
        Some("roxterm".to_string()),
        Some("termite".to_string()),
        Some("lxterminal".to_string()),
        Some("terminology".to_string()),
        Some("st".to_string()),
        Some("qterminal".to_string()),
        Some("lilyterm".to_string()),
        Some("tilix".to_string()),
        Some("terminix".to_string()),
        Some("konsole".to_string()),
        Some("kitty".to_string()),
        Some("guake".to_string()),
        Some("tilda".to_string()),
        Some("alacritty".to_string()),
        Some("hyper".to_string()),
    ];

    for terminal in terminals {
        if let Some(term) = terminal {
            return Some(match term.as_str() {
                "mate-terminal" | "gnome-terminal" | "terminator" | "xfce4-terminal"
                | "lxterminal" | "terminology" | "st" | "qterminal" | "konsole" | "kitty"
                | "guake" | "tilda" | "alacritty" | "hyper" => format!("{} --", term),
                "tilix" | "terminix" => term,
                _ => format!("{} -e ", term),
            });
        }
    }

    None
}
