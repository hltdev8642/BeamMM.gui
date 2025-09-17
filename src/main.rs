#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use beammm::Preset;
use eframe::egui;
use std::path::PathBuf;

mod components;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "BeamMM.gui",
        options,
        Box::new(|cc| {
            // --- Font setup: try runtime load from `assets/` then fall back to defaults ---
            let mut fonts = egui::FontDefinitions::default();

            // Try runtime loading (smaller binary) from `assets/YourFont-Regular.ttf`
            // Create an `assets` folder next to the executable and drop a TTF there to override.
            if let Ok(bytes) = std::fs::read("assets/YourFont-Regular.ttf") {
                fonts.font_data.insert(
                    "runtime_font".to_owned(),
                    egui::FontData::from_owned(bytes),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "runtime_font".to_owned());
            }

            // Apply fonts to the context
            cc.egui_ctx.set_fonts(fonts);

            // Tweak the style: set text style sizes
            let mut style = (*cc.egui_ctx.style()).clone();
            // Slightly smaller, less tall text styles
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::new(22.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Monospace, egui::FontId::new(11.0, egui::FontFamily::Monospace)),
                (egui::TextStyle::Button, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional)),
            ]
            .into();

            // Reduce vertical spacing slightly to make UI feel less tall
            style.spacing.item_spacing = egui::vec2(8.0, 4.0);
            cc.egui_ctx.set_style(style);

            Ok(Box::<App>::default())
        }),
    )
}

#[derive(Debug)]
struct BeamPaths {
    beamng_dir: PathBuf,
    mods_dir: PathBuf,
    beammm_dir: PathBuf,
    presets_dir: PathBuf,
}

struct StagedMod {
    mod_name: String,
    selected: bool,
    createtime: Option<i64>,
}

#[derive(Debug, PartialEq)]
pub enum SortOption {
    Name,
    Status,
    Selection,
    Date,
}

struct App {
    beam_mod_config: beammm::game::ModCfg,
    beam_paths: BeamPaths,
    beamng_version: String,
    version: String,
    staged_mods: Vec<StagedMod>,
    presets: Vec<(String, Preset)>,
    current_preset: Option<String>,
    new_preset_name: String,
    mod_search_query: String,
    sort_option: SortOption,
    sort_ascending: bool,
    filter_active_only: bool,
    filter_inactive_only: bool,
    filter_selected_only: bool,
    needs_sort: bool, // Track if sorting is needed
}

impl Default for App {
    // We will have to learn how to better handle these possible errors.
    fn default() -> Self {
        let beamng_dir = beammm::path::beamng_dir_default().unwrap();
        let beamng_version = beammm::game_version(&beamng_dir).unwrap();
        let mods_dir = beammm::path::mods_dir(&beamng_dir, &beamng_version).unwrap();
        let beammm_dir = beammm::path::beammm_dir().unwrap();
        let presets_dir = beammm::path::presets_dir(&beammm_dir).unwrap();
        let beam_paths = BeamPaths {
            beamng_dir: beamng_dir.clone(),
            mods_dir: mods_dir.clone(),
            beammm_dir,
            presets_dir,
        };
        
        let mod_cfg = beammm::game::ModCfg::load_from_path(&beam_paths.mods_dir).unwrap();
        let mut staged_mods = mod_cfg.get_mods().collect::<Vec<&String>>();
        staged_mods.sort();

        // Load db.json to get creation times
        let db_path = beam_paths.mods_dir.join("db.json");
        
        let db_content = match std::fs::read_to_string(&db_path) {
            Ok(content) => content,
            Err(_) => String::new()
        };
        
        let db: serde_json::Value = if db_content.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&db_content).unwrap_or(serde_json::Value::Null)
        };

        let staged_mods = staged_mods
            .into_iter()
            .map(|mod_name| {
                let createtime = db.get("mods")
                    .and_then(|mods| mods.get(&mod_name.clone()))
                    .and_then(|m| m.get("stat"))
                    .and_then(|s| s.get("createtime"))
                    .and_then(|t| t.as_i64());
                
                StagedMod {
                    mod_name: mod_name.to_owned(),
                    selected: false,
                    createtime,
                }
            })
            .collect();

        let presets = Preset::list(&beam_paths.presets_dir)
            .unwrap()
            .map(|preset_name| {
                (
                    preset_name.clone(),
                    Preset::load_from_path(&preset_name, &beam_paths.presets_dir).unwrap(),
                )
            })
            .collect();
        Self {
            beam_mod_config: mod_cfg,
            beam_paths,
            beamng_version,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            staged_mods,
            presets,
            current_preset: None,
            new_preset_name: String::new(),
            mod_search_query: String::new(),
            sort_option: SortOption::Name,
            sort_ascending: true,
            filter_active_only: false,
            filter_inactive_only: false,
            filter_selected_only: false,
            needs_sort: true,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        components::title_panel(ctx, self);
        components::presets_panel(ctx, self);
        components::mods_panel(ctx, self);
    }
}
