use crate::App;
use crate::SortOption;
use beammm::Preset;
use eframe::egui;
use egui::RichText;
use egui_extras::{Column, TableBuilder};

pub fn title_panel(ctx: &egui::Context, app_data: &App) {
    egui::TopBottomPanel::top("title_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("BeamMM.gui");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(&app_data.version);
                ui.label("Version: ");
                ui.separator();
                ui.label(&app_data.beamng_version);
                ui.label("BeamNG.drive: ");
            });
        });
    });
}

pub fn presets_panel(ctx: &egui::Context, app_data: &mut App) {
    egui::SidePanel::right("presets_panel").show(ctx, |ui| {
        ui.heading("Presets");
        ui.horizontal(|_| {});

        presets_table_component(ui, app_data);

        ui.separator();

        ui.horizontal(|ui| {
            let mut preset_name: String = if let Some(preset_name) = &app_data.current_preset {
                preset_name
            } else {
                "None"
            }
            .into();
            ui.label("Edit Preset:");
            preset_select_component(ui, app_data, &mut preset_name);
            app_data.current_preset = if preset_name == "None" {
                None
            } else {
                Some(preset_name)
            };
        });
        let mut delete_preset = false;
        if let Some(preset_name) = &app_data.current_preset {
            if ui.button("Delete Preset").clicked() {
                delete_preset = true;
            }

            // ui.label("Preset Mods");

            let preset = &mut app_data
                .presets
                .iter_mut()
                .find(|(name, _)| name == preset_name)
                .unwrap()
                .1;

            let mut mods_to_remove = Vec::new();

            ui.push_id("preset_mods", |ui| {
                TableBuilder::new(ui)
                    .column(Column::exact(75.0).resizable(false))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("");
                        });
                        header.col(|ui| {
                            ui.label("Preset Mods");
                        });
                    })
                    .body(|mut body| {
                        for mod_name in preset.get_mods().clone().into_iter() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    if ui.button("Remove").clicked() {
                                        mods_to_remove.push(mod_name.clone());
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(&*mod_name);
                                });
                            });
                        }
                    });
                preset.remove_mods(&mods_to_remove);
                preset
                    .save_to_path(&app_data.beam_paths.presets_dir)
                    .unwrap();
            });
        }
        if delete_preset {
            if let Some(preset_name) = &app_data.current_preset {
                Preset::delete(&preset_name, &app_data.beam_paths.presets_dir).unwrap();
                app_data.presets.retain(|(name, _)| name != preset_name);
            }
            app_data.current_preset = None;
        }
    });
}

pub fn mods_panel(ctx: &egui::Context, app_data: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Mods");
        ui.horizontal(|_| {});
        mod_actions_component(ui, app_data);
        mods_table_component(ui, app_data);
    });
}

fn preset_select_component(ui: &mut egui::Ui, app_data: &mut App, preset_name: &mut String) {
    ui.menu_button(preset_name.clone(), |ui| {
        for preset in beammm::Preset::list(&app_data.beam_paths.presets_dir).unwrap() {
            if ui.button(&preset).clicked() {
                *preset_name = preset.to_owned();
                ui.close_menu();
            }
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut app_data.new_preset_name);
            if ui.button("Create").clicked() {
                let new_preset_name = app_data.new_preset_name.clone();
                app_data.new_preset_name = "".into();
                let new_preset = Preset::new(new_preset_name.clone(), vec![]);
                new_preset
                    .save_to_path(&app_data.beam_paths.presets_dir)
                    .unwrap();
                app_data.presets.push((new_preset_name.clone(), new_preset));
                *preset_name = new_preset_name;
                ui.close_menu();
            }
        })
    });
}

fn presets_table_component(ui: &mut egui::Ui, app_data: &mut App) {
    ui.label("All Presets:");
    TableBuilder::new(ui)
        .column(Column::exact(75.0))
        .column(Column::auto().resizable(false))
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.add(egui::Label::new("Enabled").wrap_mode(egui::TextWrapMode::Extend));
            });
            header.col(|ui| {
                ui.label("Preset Name");
            });
        })
        .body(|mut body| {
            for (preset_name, preset) in &mut app_data.presets {
                body.row(20.0, |mut row| {
                    row.col(|ui| {
                        let text = if preset.is_enabled() {
                            RichText::new("Enabled").color(egui::Color32::GREEN)
                        } else {
                            RichText::new("Disabled").color(egui::Color32::RED)
                        };
                        if ui.button(text).clicked() {
                            if preset.is_enabled() {
                                preset.disable(&mut app_data.beam_mod_config).unwrap();
                            } else {
                                preset.enable();
                            }
                            preset
                                .save_to_path(&app_data.beam_paths.presets_dir)
                                .unwrap();
                            app_data
                                .beam_mod_config
                                .apply_presets(&app_data.beam_paths.presets_dir)
                                .unwrap();
                            app_data
                                .beam_mod_config
                                .save_to_path(&app_data.beam_paths.mods_dir)
                                .unwrap();
                        }
                    });
                    row.col(|ui| {
                        ui.label(&*preset_name);
                    });
                });
            }
        });
}

fn mods_table_component(ui: &mut egui::Ui, app_data: &mut App) {
    // Add search bar    // Search and filter controls
    ui.horizontal(|ui| {
        ui.label("Search mods: ");
        ui.text_edit_singleline(&mut app_data.mod_search_query);
        
        ui.separator();
        
        // Filter options
        if ui.selectable_label(app_data.filter_active_only, "Active Only").clicked() {
            app_data.filter_active_only = !app_data.filter_active_only;
            app_data.filter_inactive_only = false;
        }
        if ui.selectable_label(app_data.filter_inactive_only, "Inactive Only").clicked() {
            app_data.filter_inactive_only = !app_data.filter_inactive_only;
            app_data.filter_active_only = false;
        }
        if ui.selectable_label(app_data.filter_selected_only, "Selected Only").clicked() {
            app_data.filter_selected_only = !app_data.filter_selected_only;
        }
    });
      // Sort controls
    ui.horizontal(|ui| {
        ui.label(RichText::new("Sort by:"));
        
        // Direction toggle
        if ui.button(if app_data.sort_ascending { "↑" } else { "↓" }).clicked() {
            app_data.sort_ascending = !app_data.sort_ascending;
            app_data.needs_sort = true;
        }        // Sort options
        ui.horizontal(|ui| {
            if ui.selectable_label(app_data.sort_option == SortOption::Name, "Name").clicked() {
                app_data.sort_option = SortOption::Name;
                app_data.needs_sort = true;
            }
            if ui.selectable_label(app_data.sort_option == SortOption::Status, "Status").clicked() {
                app_data.sort_option = SortOption::Status;
                app_data.needs_sort = true;
            }
            if ui.selectable_label(app_data.sort_option == SortOption::Selection, "Selection").clicked() {
                app_data.sort_option = SortOption::Selection;
                app_data.needs_sort = true;
            }
            if ui.selectable_label(app_data.sort_option == SortOption::Date, "Date Added").clicked() {
                app_data.sort_option = SortOption::Date;
                app_data.needs_sort = true;
            }
        });
          // Apply sorting only when needed
        if app_data.needs_sort {
            app_data.staged_mods.sort_by(|a, b| {
                let cmp = match app_data.sort_option {
                    SortOption::Name => a.mod_name.cmp(&b.mod_name),
                    SortOption::Status => {
                        let a_active = app_data.beam_mod_config.is_mod_active(&a.mod_name).unwrap();
                        let b_active = app_data.beam_mod_config.is_mod_active(&b.mod_name).unwrap();
                        a_active.cmp(&b_active)
                    },
                    SortOption::Selection => a.selected.cmp(&b.selected),
                    SortOption::Date => match (a.createtime, b.createtime) {
                        (Some(a_time), Some(b_time)) => a_time.cmp(&b_time),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.mod_name.cmp(&b.mod_name), // fallback to name sorting
                    },
                };
                
                if app_data.sort_ascending {
                    cmp
                } else {
                    cmp.reverse()
                }
            });
            app_data.needs_sort = false;
        }
    });

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().resizable(false))
        .column(Column::exact(75.0).resizable(false))
        .column(Column::remainder())        .header(20.0, |mut header| {
            header.col(|ui| {
                let text = if app_data.sort_option == SortOption::Selection {
                    if app_data.sort_ascending {
                        RichText::new("Select ↑")
                    } else {
                        RichText::new("Select ↓")
                    }
                } else {
                    RichText::new("Select")
                };
                ui.label(text);
            });
            header.col(|ui| {
                let text = if app_data.sort_option == SortOption::Status {
                    if app_data.sort_ascending {
                        RichText::new("Active ↑")
                    } else {
                        RichText::new("Active ↓")
                    }
                } else {
                    RichText::new("Active")
                };
                ui.label(text);
            });            header.col(|ui| {
                let text = if app_data.sort_option == SortOption::Name {
                    if app_data.sort_ascending {
                        RichText::new("Mod Name ↑")
                    } else {
                        RichText::new("Mod Name ↓")
                    }
                } else if app_data.sort_option == SortOption::Date {
                    if app_data.sort_ascending {
                        RichText::new("Mod Name (Date ↑)")
                    } else {
                        RichText::new("Mod Name (Date ↓)")
                    }
                } else {
                    RichText::new("Mod Name")
                };
                ui.label(text);
            });
        })
        .body(|mut body| {            // First collect mod statuses
            let mod_statuses: Vec<_> = app_data.staged_mods.iter().map(|m| (
                m.mod_name.clone(),
                app_data.beam_mod_config.is_mod_active(&m.mod_name).unwrap()
            )).collect();

            let filtered_mods = app_data.staged_mods.iter_mut().enumerate().filter(|(i, m)| {
                // Text search filter
                let text_matches = m.mod_name
                    .to_lowercase()
                    .contains(&app_data.mod_search_query.to_lowercase());
                
                // Active/Inactive filter
                let active_status = mod_statuses[*i].1;
                let status_matches = if app_data.filter_active_only {
                    active_status
                } else if app_data.filter_inactive_only {
                    !active_status
                } else {
                    true
                };
                
                // Selected filter
                let selected_matches = if app_data.filter_selected_only {
                    m.selected
                } else {
                    true
                };

                text_matches && status_matches && selected_matches
            }).map(|(_, m)| m);

            for staged_mod in filtered_mods {
                body.row(20.0, |mut row| {                    row.col(|ui| {
                        ui.checkbox(&mut staged_mod.selected, "");
                    });
                    row.col(|ui| {
                        let active = app_data
                            .beam_mod_config
                            .is_mod_active(&staged_mod.mod_name)
                            .unwrap();
                        let text = if active {
                            RichText::new("Active").color(egui::Color32::from_rgb(50, 200, 50))
                        } else {
                            RichText::new("Inactive").color(egui::Color32::from_rgb(200, 50, 50))
                        };
                        if ui.button(text).clicked() {
                            app_data
                                .beam_mod_config
                                .set_mod_active(&staged_mod.mod_name, !active)
                                .unwrap();
                            app_data
                                .beam_mod_config
                                .save_to_path(&app_data.beam_paths.mods_dir)
                                .unwrap();
                        }
                    });
                    row.col(|ui| {
                        ui.label(&staged_mod.mod_name);
                    });
                });
            }
        });
}

/// Buttons to select/deselect/enabled/disable mods etc.
/// Displayed right above the mods table.
fn mod_actions_component(ui: &mut egui::Ui, app_data: &mut App) {    ui.horizontal(|ui| {
    if ui.button(RichText::new("Select All").size(12.0)).clicked() {
            for staged_mod in &mut app_data.staged_mods {
                if staged_mod.mod_name
                    .to_lowercase()
                    .contains(&app_data.mod_search_query.to_lowercase())
                {
                    staged_mod.selected = true;
                }
            }
        }

    if ui.button(RichText::new("Deselect All").size(12.0)).clicked() {
            for staged_mod in &mut app_data.staged_mods {
                staged_mod.selected = false;
            }
        }
    });    ui.horizontal(|ui| {
            if ui.button(
            RichText::new("Enable Selected")
                .size(12.0)
                .color(egui::Color32::from_rgb(50, 200, 50)),
        ).clicked() {
            for staged_mod in &app_data.staged_mods {
                if staged_mod.selected {
                    app_data
                        .beam_mod_config
                        .set_mod_active(&staged_mod.mod_name, true)
                        .unwrap();
                }
            }
            app_data
                .beam_mod_config
                .save_to_path(&app_data.beam_paths.mods_dir)
                .unwrap();
        }

        if ui.button(
            RichText::new("Disable Selected")
                .size(12.0)
                .color(egui::Color32::from_rgb(200, 50, 50)),
        ).clicked() {
            for staged_mod in &app_data.staged_mods {
                if staged_mod.selected {
                    app_data
                        .beam_mod_config
                        .set_mod_active(&staged_mod.mod_name, false)
                        .unwrap();
                }
            }
            app_data
                .beam_mod_config
                .apply_presets(&app_data.beam_paths.presets_dir)
                .unwrap();
            app_data
                .beam_mod_config
                .save_to_path(&app_data.beam_paths.mods_dir)
                .unwrap();
        }
    });    if let Some(preset_name) = &app_data.current_preset {
        ui.horizontal(|ui| {
            if ui.button(
                RichText::new(format!("Add to Preset '{}'", preset_name))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(50, 150, 200)),
            ).clicked() {
                let preset = &mut app_data
                    .presets
                    .iter_mut()
                    .find(|(name, _)| name == preset_name)
                    .unwrap()
                    .1;
                for staged_mod in &app_data.staged_mods {
                    if staged_mod.selected {
                        preset.add_mod(&staged_mod.mod_name);
                    }
                }
                preset
                    .save_to_path(&app_data.beam_paths.presets_dir)
                    .unwrap();
                app_data
                    .beam_mod_config
                    .apply_presets(&app_data.beam_paths.presets_dir)
                    .unwrap();
            }
        });
    }
}
