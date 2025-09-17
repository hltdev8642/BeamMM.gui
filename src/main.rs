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
            match std::fs::read("assets/YourFont-Regular.ttf") {
                Ok(bytes) => {
                    fonts.font_data.insert(
                        "runtime_font".to_owned(),
                        egui::FontData::from_owned(bytes),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "runtime_font".to_owned());
                    eprintln!("Loaded runtime font from assets/YourFont-Regular.ttf");
                }
                Err(_) => {
                    eprintln!("Runtime font not found at assets/YourFont-Regular.ttf; using default fonts");
                }
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
            cc.egui_ctx.set_style(style.clone());

            // Log applied text style sizes to help debug font/style issues
            let heading_size = style
                .text_styles
                .get(&egui::TextStyle::Heading)
                .map(|f| f.size)
                .unwrap_or(0.0);
            let body_size = style
                .text_styles
                .get(&egui::TextStyle::Body)
                .map(|f| f.size)
                .unwrap_or(0.0);
            eprintln!("Applied style text sizes: heading={} body={}", heading_size, body_size);

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
    // Optional metadata from db.json
    filename: Option<String>,
    fullpath: Option<String>,
    mod_type: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum SortOption {
    Name,
    Status,
    Selection,
    Date,
    Filename,
    Fullpath,
    ModType,
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
    // Filters for new db.json fields
    filename_filter: String,
    fullpath_filter: String,
    mod_type_filter: String,
    available_mod_types: Vec<String>,
    sort_option: SortOption,
    sort_ascending: bool,
    filter_active_only: bool,
    filter_inactive_only: bool,
    filter_selected_only: bool,
    needs_sort: bool, // Track if sorting is needed
    advanced_filters_open: bool,
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

        let staged_mods: Vec<StagedMod> = staged_mods
            .into_iter()
            .map(|mod_name| {
                // Look up the entry under the "mods" object and pull optional fields
                let entry = db.get("mods").and_then(|mods| mods.get(&mod_name.clone()));

                let createtime = entry
                    .and_then(|m| m.get("stat"))
                    .and_then(|s| s.get("createtime"))
                    .and_then(|t| t.as_i64());

                let filename = entry
                    .and_then(|m| m.get("filename"))
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_owned());

                let fullpath = entry
                    .and_then(|m| m.get("fullpath"))
                    .and_then(|f| f.as_str())
                    .map(|s| s.to_owned());

                // Some db.json use "modType" or "modtype" etc — try a few variants
                let mod_type = entry
                    .and_then(|m| m.get("modType").or_else(|| m.get("modtype")).or_else(|| m.get("type")))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_owned());

                StagedMod {
                    mod_name: mod_name.to_owned(),
                    selected: false,
                    createtime,
                    filename,
                    fullpath,
                    mod_type,
                }
            })
            .collect();
        // Compute available mod types before we move staged_mods into the App struct
        let mut available_mod_types: Vec<String> = staged_mods
            .iter()
            .filter_map(|m| m.mod_type.clone())
            .filter(|s| !s.is_empty())
            .collect();
        available_mod_types.sort();
        available_mod_types.dedup();

        let presets = Preset::list(&beam_paths.presets_dir)
            .unwrap()
            .map(|preset_name| {
                (
                    preset_name.clone(),
                    Preset::load_from_path(&preset_name, &beam_paths.presets_dir).unwrap(),
                )
            })
            .collect();
        let advanced_filters_open = false; // Default value for advanced_filters_open
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
            filename_filter: String::new(),
            fullpath_filter: String::new(),
            mod_type_filter: String::new(),
            // Use precomputed available_mod_types
            available_mod_types,
            sort_option: SortOption::Name,
            sort_ascending: true,
            filter_active_only: false,
            filter_inactive_only: false,
            filter_selected_only: false,
            needs_sort: true,
            advanced_filters_open,
        }
    }
}

impl App {
    fn save_gui_config(&self) {
        let gui_config_path = self.beam_paths.beammm_dir.join("gui_config.json");
        let cfg = serde_json::json!({
            "advanced_filters_open": self.advanced_filters_open,
        });
        if let Err(e) = std::fs::write(&gui_config_path, serde_json::to_string_pretty(&cfg).unwrap()) {
            eprintln!("Failed to write gui config {}: {}", gui_config_path.display(), e);
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
