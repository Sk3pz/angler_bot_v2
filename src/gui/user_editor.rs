use eframe::egui;
use serenity::all::UserId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serenity::http::Http;
use crate::data_management::userfile::UserFile;
use crate::fishing::shop::Shop; 
use crate::fishing::Attribute; 
use crate::fishing::rod_data::{
    bait::{Bait, BaitAttraction, BaitBias, AttractionQuality, BaitPotency},
    rods::RodBase,
    reels::Reel,
    lines::Line,
    sinkers::Sinker
};
use crate::fishing::fish_data::{fish::FishCategory, rarity::FishRarity};

pub struct UserEditor {
    http: Option<Arc<Http>>,
    user_ids: Vec<UserId>,
    usernames: Arc<Mutex<HashMap<UserId, String>>>,
    selected_user_id: Option<UserId>,
    selected_user_file: Option<UserFile>,
    search_query: String,
    
    // Item adding/editing state
    editing_item: Option<EditingItem>,
    
    status_message: Option<(String, std::time::Instant)>,

    // Shop Catalog for reference
    shop: Shop,
}

#[derive(Clone, Debug)]
enum EditingItem {
    // Bait
    NewBait(Bait),
    ExistingBait { index: usize, bait: Bait },
    
    // Rods
    NewRod(RodBase),
    ExistingRod { index: usize, val: RodBase },
    
    // Reels
    NewReel(Reel),
    ExistingReel { index: usize, val: Reel },
    
    // Lines
    NewLine(Line),
    ExistingLine { index: usize, val: Line },
    
    // Sinkers
    NewSinker(Sinker),
    ExistingSinker { index: usize, val: Sinker },
}

impl UserEditor {
    pub fn new(http: Option<Arc<Http>>) -> Self {
        Self {
            http,
            user_ids: Vec::new(),
            usernames: Arc::new(Mutex::new(HashMap::new())),
            selected_user_id: None,
            selected_user_file: None,
            search_query: String::new(),
            editing_item: None,
            status_message: None,
            shop: Shop::load(), // Load static game data
        }
    }

    pub fn refresh_users(&mut self) {
        // Scan data/users directory
        self.user_ids.clear();
        if let Ok(entries) = std::fs::read_dir("./data/users") {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(id) = file_stem.parse::<u64>() {
                            self.user_ids.push(UserId::new(id));
                        }
                    }
                }
            }
        }
        
        // Resolve usernames if http is available
        if let Some(http) = &self.http {
            let http = http.clone();
            let usernames = self.usernames.clone();
            let ids = self.user_ids.clone();

            tokio::spawn(async move {
                for id in ids {
                    if {
                         let lock = usernames.lock().unwrap();
                         !lock.contains_key(&id)
                    } {
                        if let Ok(user) = id.to_user(&http).await {
                             let mut lock = usernames.lock().unwrap();
                             lock.insert(id, user.name);
                        }
                    }
                }
            });
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // SIDEBAR: User List
        egui::SidePanel::left("user_list_panel")
            .resizable(false)
            .exact_width(220.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.heading("Users");
                    if ui.button("Refresh List").clicked() || self.user_ids.is_empty() {
                        self.refresh_users();
                    }
                    ui.separator();
                    ui.text_edit_singleline(&mut self.search_query);
                    ui.separator();

                    let usernames = self.usernames.lock().unwrap().clone();
                    egui::ScrollArea::vertical()
                        .id_salt("user_list_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                        for (idx, id) in self.user_ids.iter().enumerate() {
                            ui.push_id(idx, |ui| { 
                                let name = usernames.get(id).cloned().unwrap_or_else(|| id.to_string());
                                
                                if !self.search_query.is_empty() && !name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                                    return;
                                }

                                if ui.selectable_label(self.selected_user_id == Some(*id), name).clicked() {
                                    self.selected_user_id = Some(*id);
                                    self.selected_user_file = Some(UserFile::read(id));
                                    self.editing_item = None; 
                                }
                            });
                        }
                    });
                });
            });

        // MAIN CONTENT AREA
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                let mut user_file_opt = self.selected_user_file.take();
                
                if let Some(user_file) = &mut user_file_opt {
                    ui.heading(format!("Editing User: {}", user_file.user_id));
                    ui.horizontal(|ui| {
                       ui.label("Balance ($):");
                       let mut dollar_val = user_file.file.balance.amount_x100 as f32 / 100.0;
                       if ui.add(egui::DragValue::new(&mut dollar_val).speed(0.1).prefix("$")).changed() {
                            user_file.file.balance.amount_x100 = (dollar_val * 100.0).round() as u32;
                       }

                       if ui.button("Save User").clicked() {
                           user_file.update();
                           self.status_message = Some(("User saved!".to_string(), std::time::Instant::now()));
                       }
                    });
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {

                        if let Some(mut editing) = self.editing_item.clone() {
                            self.show_item_editor(ui, &mut editing, user_file);
                            
                            if self.editing_item.is_some() {
                                self.editing_item = Some(editing);
                            }
                        } else {
                            self.show_inventory_list(ui, user_file);
                        }
                    });

                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Select a user from the left to start editing.");
                    });
                }
                
                self.selected_user_file = user_file_opt;
                
                if let Some((msg, time)) = &self.status_message {
                   if time.elapsed().as_secs() < 3 {
                       ui.label(msg);
                   }
               }
        });
    }

    fn show_inventory_list(&mut self, ui: &mut egui::Ui, user_file: &mut UserFile) {
            ui.heading("Inventory");
            
            ui.collapsing("Bait Bucket", |ui| {
                if ui.button("+ Add New Custom Bait").clicked() {
                    let default_bait = Bait::generate(BaitPotency::Low, false);
                    self.editing_item = Some(EditingItem::NewBait(default_bait));
                }
                
                ui.separator();
                
                let mut index_to_edit = None;
                let mut index_to_delete = None;

                for (i, bait) in user_file.file.inventory.bait_bucket.baits.iter().enumerate() {
                    ui.push_id(format!("bait_row_{}", i), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", i+1, bait.name));
                            if ui.button("Edit").clicked() {
                                index_to_edit = Some(i);
                            }
                            if ui.button("Del").clicked() {
                                index_to_delete = Some(i);
                            }
                        });
                    });
                }
                
                if let Some(i) = index_to_edit {
                    if let Some(bait) = user_file.file.inventory.bait_bucket.get(i) {
                         self.editing_item = Some(EditingItem::ExistingBait { index: i, bait: bait.clone() });
                    }
                }
                if let Some(i) = index_to_delete {
                     user_file.file.inventory.bait_bucket.remove_index(i);
                }
            });

             ui.collapsing("Rods", |ui| {
                 ui.horizontal(|ui| {
                    if ui.button("+ Add New Rod").clicked() {
                        let new_rod = RodBase {
                            name: "New Rod".to_string(),
                            description: "".to_string(),
                            price: 0.0,
                            sensitivity: 1.0,
                            strength_bonus: 1.0,
                            efficiency_multiplier: 1.0,
                        };
                        self.editing_item = Some(EditingItem::NewRod(new_rod));
                    }
                    
                    // SHOP CATALOG BUTTON
                    ui.menu_button("✚ Shop Catalog", |ui| {
                         egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                             for item in &self.shop.rods {
                                 if ui.button(&item.name).on_hover_text(&item.description).clicked() {
                                     user_file.file.inventory.rods.push(item.clone());
                                     user_file.update();
                                     self.status_message = Some((format!("Added {}!", item.name), std::time::Instant::now()));
                                     ui.close_menu();
                                 }
                             }
                         });
                    });
                 });
                 ui.separator();

                 let mut index_to_edit = None;
                 let mut index_to_delete = None;
                 for (i, item) in user_file.file.inventory.rods.iter().enumerate() {
                     ui.push_id(format!("rod_row_{}", i), |ui| {
                         ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", i+1, item.name));
                            if ui.button("Edit").clicked() { index_to_edit = Some(i); }
                            if ui.button("Del").clicked() { index_to_delete = Some(i); }
                         });
                     });
                 }
                 if let Some(i) = index_to_edit {
                     if let Some(val) = user_file.file.inventory.rods.get(i) {
                         self.editing_item = Some(EditingItem::ExistingRod { index: i, val: val.clone() });
                     }
                 }
                  if let Some(i) = index_to_delete { user_file.file.inventory.rods.remove(i); }
             });
             
             ui.collapsing("Reels", |ui| {
                 ui.horizontal(|ui| {
                     if ui.button("+ Add New Reel").clicked() {
                         let new_reel = Reel {
                             name: "New Reel".to_string(),
                             description: "".to_string(),
                             price: 0.0,
                             speed_multiplier: 1.0,
                         };
                         self.editing_item = Some(EditingItem::NewReel(new_reel));
                     }
                     
                     // SHOP CATALOG BUTTON
                     ui.menu_button("✚ Shop Catalog", |ui| {
                         egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                             for item in &self.shop.reels {
                                 if ui.button(&item.name).on_hover_text(&item.description).clicked() {
                                     user_file.file.inventory.reels.push(item.clone());
                                     user_file.update();
                                     self.status_message = Some((format!("Added {}!", item.name), std::time::Instant::now()));
                                     ui.close_menu();
                                 }
                             }
                         });
                    });
                 });
                 ui.separator();

                 let mut index_to_edit = None;
                 let mut index_to_delete = None;
                 for (i, item) in user_file.file.inventory.reels.iter().enumerate() {
                     ui.push_id(format!("reel_row_{}", i), |ui| {
                         ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", i+1, item.name));
                             if ui.button("Edit").clicked() { index_to_edit = Some(i); }
                            if ui.button("Del").clicked() { index_to_delete = Some(i); }
                         });
                     });
                 }
                if let Some(i) = index_to_edit {
                     if let Some(val) = user_file.file.inventory.reels.get(i) {
                         self.editing_item = Some(EditingItem::ExistingReel { index: i, val: val.clone() });
                     }
                 }
                 if let Some(i) = index_to_delete { user_file.file.inventory.reels.remove(i); }
             });

             ui.collapsing("Lines", |ui| {
                 ui.horizontal(|ui| {
                     if ui.button("+ Add New Line").clicked() {
                         let new_line = Line {
                             name: "New Line".to_string(),
                             description: "".to_string(),
                             price: 0.0,
                             strength: 10,
                         };
                         self.editing_item = Some(EditingItem::NewLine(new_line));
                     }
                     
                     // SHOP CATALOG BUTTON
                     ui.menu_button("✚ Shop Catalog", |ui| {
                         egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                             for item in &self.shop.lines {
                                 if ui.button(&item.name).on_hover_text(&item.description).clicked() {
                                     user_file.file.inventory.lines.push(item.clone());
                                     user_file.update();
                                     self.status_message = Some((format!("Added {}!", item.name), std::time::Instant::now()));
                                     ui.close_menu();
                                 }
                             }
                         });
                    });
                 });
                 ui.separator();

                 let mut index_to_edit = None;
                  let mut index_to_delete = None;
                 for (i, item) in user_file.file.inventory.lines.iter().enumerate() {
                    ui.push_id(format!("line_row_{}", i), |ui| {
                         ui.horizontal(|ui| {
                             ui.label(format!("{}. {}", i+1, item.name));
                              if ui.button("Edit").clicked() { index_to_edit = Some(i); }
                             if ui.button("Del").clicked() { index_to_delete = Some(i); }
                         });
                    });
                 }
                if let Some(i) = index_to_edit {
                     if let Some(val) = user_file.file.inventory.lines.get(i) {
                         self.editing_item = Some(EditingItem::ExistingLine { index: i, val: val.clone() });
                     }
                 }
                 if let Some(i) = index_to_delete { user_file.file.inventory.lines.remove(i); }
             });

             ui.collapsing("Sinkers", |ui| {
                 ui.horizontal(|ui| {
                     if ui.button("+ Add New Sinker").clicked() {
                         let new_sinker = Sinker {
                             name: "New Sinker".to_string(),
                             description: "".to_string(),
                             price: 0.0,
                             weight: 1.0,
                             depth_range: Attribute {
                                 min: 0.0, max: 10.0, average: 5.0
                             },
                         };
                         self.editing_item = Some(EditingItem::NewSinker(new_sinker));
                     }
                     
                     // SHOP CATALOG BUTTON
                     ui.menu_button("✚ Shop Catalog", |ui| {
                         egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                             for item in &self.shop.sinkers {
                                 if ui.button(&item.name).on_hover_text(&item.description).clicked() {
                                     user_file.file.inventory.sinkers.push(item.clone());
                                     user_file.update();
                                     self.status_message = Some((format!("Added {}!", item.name), std::time::Instant::now()));
                                     ui.close_menu();
                                 }
                             }
                         });
                    });
                 });
                 ui.separator();

                 let mut index_to_edit = None;
                 let mut index_to_delete = None;
                 for (i, item) in user_file.file.inventory.sinkers.iter().enumerate() {
                      ui.push_id(format!("sink_row_{}", i), |ui| {
                         ui.horizontal(|ui| {
                             ui.label(format!("{}. {}", i+1, item.name));
                             if ui.button("Edit").clicked() { index_to_edit = Some(i); }
                             if ui.button("Del").clicked() { index_to_delete = Some(i); }
                         });
                      });
                 }
                 if let Some(i) = index_to_edit {
                     if let Some(val) = user_file.file.inventory.sinkers.get(i) {
                         self.editing_item = Some(EditingItem::ExistingSinker { index: i, val: val.clone() });
                     }
                 }
                 if let Some(i) = index_to_delete { user_file.file.inventory.sinkers.remove(i); }
             });
    }

    fn show_item_editor(&mut self, ui: &mut egui::Ui, editing: &mut EditingItem, user_file: &mut UserFile) {
        ui.horizontal(|ui| {
             if ui.button("⬅ Back to Inventory").clicked() {
                 self.editing_item = None;
                 return;
             }
             ui.heading("Item Editor");
        });
        ui.separator();
        
        if self.editing_item.is_none() { 
            return; 
        }

        match editing {
            // BAIT
            EditingItem::NewBait(bait) | EditingItem::ExistingBait { bait, .. } => {
                self.edit_bait(ui, bait);
                ui.separator();
                if ui.button("Save to Inventory").clicked() {
                    match editing {
                        EditingItem::NewBait(b) => {
                            user_file.file.inventory.bait_bucket.add(b.clone());
                            self.status_message = Some(("Created new bait!".to_string(), std::time::Instant::now()));
                        },
                        EditingItem::ExistingBait { index, bait } => {
                             if let Some(existing) = user_file.file.inventory.bait_bucket.baits.get_mut(*index) {
                                 *existing = bait.clone();
                                 self.status_message = Some(("Updated bait!".to_string(), std::time::Instant::now()));
                             }
                        },
                         _ => {}
                    }
                    user_file.update(); 
                    self.editing_item = None; 
                }
            },
            
            // RODS
            EditingItem::NewRod(val) | EditingItem::ExistingRod { val, .. } => {
                 ui.horizontal(|ui| { ui.label("Name"); ui.text_edit_singleline(&mut val.name); });
                 ui.horizontal(|ui| { ui.label("Desc"); ui.text_edit_singleline(&mut val.description); });
                 ui.horizontal(|ui| { ui.label("Price"); ui.add(egui::DragValue::new(&mut val.price)); });
                 ui.horizontal(|ui| { ui.label("Sensitivity"); ui.add(egui::DragValue::new(&mut val.sensitivity).speed(0.01)); });
                 ui.horizontal(|ui| { ui.label("Strength Bonus"); ui.add(egui::DragValue::new(&mut val.strength_bonus).speed(0.01)); });
                 ui.horizontal(|ui| { ui.label("Efficiency"); ui.add(egui::DragValue::new(&mut val.efficiency_multiplier).speed(0.01)); });
                 
                 ui.separator();
                 if ui.button("Save Rod").clicked() {
                     match editing {
                        EditingItem::NewRod(v) => {
                            user_file.file.inventory.rods.push(v.clone());
                            self.status_message = Some(("Created new rod!".to_string(), std::time::Instant::now()));
                        },
                        EditingItem::ExistingRod { index, val } => {
                            if let Some(existing) = user_file.file.inventory.rods.get_mut(*index) {
                                *existing = val.clone();
                                self.status_message = Some(("Updated rod!".to_string(), std::time::Instant::now()));
                            }
                        },
                        _ => {}
                     }
                     user_file.update();
                     self.editing_item = None;
                 }
            },

            // REELS
            EditingItem::NewReel(val) | EditingItem::ExistingReel { val, .. } => {
                 ui.horizontal(|ui| { ui.label("Name"); ui.text_edit_singleline(&mut val.name); });
                 ui.horizontal(|ui| { ui.label("Desc"); ui.text_edit_singleline(&mut val.description); });
                 ui.horizontal(|ui| { ui.label("Price"); ui.add(egui::DragValue::new(&mut val.price)); });
                 ui.horizontal(|ui| { ui.label("Speed Mult"); ui.add(egui::DragValue::new(&mut val.speed_multiplier).speed(0.01)); });

                 ui.separator();
                 if ui.button("Save Reel").clicked() {
                     match editing {
                        EditingItem::NewReel(v) => {
                             user_file.file.inventory.reels.push(v.clone());
                             self.status_message = Some(("Created new reel!".to_string(), std::time::Instant::now()));
                        },
                        EditingItem::ExistingReel { index, val } => {
                              if let Some(existing) = user_file.file.inventory.reels.get_mut(*index) {
                                  *existing = val.clone();
                                  self.status_message = Some(("Updated reel!".to_string(), std::time::Instant::now()));
                              }
                        },
                        _ => {}
                     }
                     user_file.update();
                     self.editing_item = None;
                 }
            },

            // LINES
             EditingItem::NewLine(val) | EditingItem::ExistingLine { val, .. } => {
                 ui.horizontal(|ui| { ui.label("Name"); ui.text_edit_singleline(&mut val.name); });
                 ui.horizontal(|ui| { ui.label("Desc"); ui.text_edit_singleline(&mut val.description); });
                 ui.horizontal(|ui| { ui.label("Price"); ui.add(egui::DragValue::new(&mut val.price)); });
                 ui.horizontal(|ui| { ui.label("Strength (lbs)"); ui.add(egui::DragValue::new(&mut val.strength)); });

                 ui.separator();
                 if ui.button("Save Line").clicked() {
                      match editing {
                        EditingItem::NewLine(v) => {
                             user_file.file.inventory.lines.push(v.clone());
                             self.status_message = Some(("Created new line!".to_string(), std::time::Instant::now()));
                        },
                        EditingItem::ExistingLine { index, val } => {
                             if let Some(existing) = user_file.file.inventory.lines.get_mut(*index) {
                                  *existing = val.clone();
                                  self.status_message = Some(("Updated line!".to_string(), std::time::Instant::now()));
                             }
                        },
                        _ => {}
                     }
                     user_file.update();
                     self.editing_item = None;
                 }
            },

            // SINKERS
            EditingItem::NewSinker(val) | EditingItem::ExistingSinker { val, .. } => {
                 ui.horizontal(|ui| { ui.label("Name"); ui.text_edit_singleline(&mut val.name); });
                 ui.horizontal(|ui| { ui.label("Desc"); ui.text_edit_singleline(&mut val.description); });
                 ui.horizontal(|ui| { ui.label("Price"); ui.add(egui::DragValue::new(&mut val.price)); });
                 ui.horizontal(|ui| { ui.label("Weight"); ui.add(egui::DragValue::new(&mut val.weight).speed(0.1)); });
                 ui.separator();
                 ui.label("Depth Range:");
                 ui.horizontal(|ui| { ui.label("Min"); ui.add(egui::DragValue::new(&mut val.depth_range.min)); });
                 ui.horizontal(|ui| { ui.label("Max"); ui.add(egui::DragValue::new(&mut val.depth_range.max)); });
                 ui.horizontal(|ui| { ui.label("Avg"); ui.add(egui::DragValue::new(&mut val.depth_range.average)); });

                 ui.separator();
                 if ui.button("Save Sinker").clicked() {
                      match editing {
                        EditingItem::NewSinker(v) => {
                             user_file.file.inventory.sinkers.push(v.clone());
                             self.status_message = Some(("Created new sinker!".to_string(), std::time::Instant::now()));
                        },
                        EditingItem::ExistingSinker { index, val } => {
                              if let Some(existing) = user_file.file.inventory.sinkers.get_mut(*index) {
                                  *existing = val.clone();
                                  self.status_message = Some(("Updated sinker!".to_string(), std::time::Instant::now()));
                              }
                        },
                        _ => {}
                     }
                     user_file.update();
                     self.editing_item = None;
                 }
            },
        }
    }
    
    fn edit_bait(&self, ui: &mut egui::Ui, bait: &mut Bait) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut bait.name);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.text_edit_multiline(&mut bait.description);
        });
        ui.horizontal(|ui| {
            ui.label("Price:");
             ui.add(egui::DragValue::new(&mut bait.price).speed(0.1));
        });
        ui.checkbox(&mut bait.reusable, "Reusable (Lure)");

        ui.separator();
        ui.heading("Attractions");
        
        let mut remove_idx = None;
        
        for (i, attr) in bait.attraction.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Attraction #{}", i+1));
                        if ui.button("Remove").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                     match attr {
                        BaitAttraction::Heavy { bias, quality } => {
                            ui.label("Type: Heavy");
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                         BaitAttraction::Light { bias, quality } => {
                            ui.label("Type: Light");
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                         BaitAttraction::Large { bias, quality } => {
                            ui.label("Type: Large");
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                         BaitAttraction::Small { bias, quality } => {
                            ui.label("Type: Small");
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                         BaitAttraction::SpecificFish { name, bias, quality } => {
                            ui.label("Type: Specific Fish");
                            ui.text_edit_singleline(name);
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                        BaitAttraction::Rarity(rarity, bias, quality) => {
                            ui.label("Type: Rarity");
                            self.edit_rarity(ui, rarity, i);
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                        BaitAttraction::Category(cat, bias, quality) => {
                            ui.label("Type: Category");
                            self.edit_category(ui, cat, i);
                            self.edit_bias(ui, bias, i);
                            self.edit_quality(ui, quality, i);
                        },
                    }
                });
            });
        }
        
        if let Some(i) = remove_idx {
            bait.attraction.remove(i);
        }
        
        ui.menu_button("Add Attraction", |ui| {
            if ui.button("Heavy").clicked() {
                bait.attraction.push(BaitAttraction::Heavy { bias: BaitBias::Low, quality: AttractionQuality::Bad });
                ui.close();
            }
            if ui.button("Light").clicked() {
                bait.attraction.push(BaitAttraction::Light { bias: BaitBias::Low, quality: AttractionQuality::Bad });
                 ui.close();
            }
            if ui.button("Large").clicked() {
                bait.attraction.push(BaitAttraction::Large { bias: BaitBias::Low, quality: AttractionQuality::Bad });
                 ui.close();
            }
            if ui.button("Small").clicked() {
                 bait.attraction.push(BaitAttraction::Small { bias: BaitBias::Low, quality: AttractionQuality::Bad });
                 ui.close();
            }
             if ui.button("Category").clicked() {
                 bait.attraction.push(BaitAttraction::Category(FishCategory::BaitFish, BaitBias::Low, AttractionQuality::Bad));
                 ui.close();
            }
             if ui.button("Rarity").clicked() {
                 bait.attraction.push(BaitAttraction::Rarity(FishRarity::Common, BaitBias::Low, AttractionQuality::Bad));
                 ui.close();
            }
             if ui.button("Specific Fish").clicked() {
                 bait.attraction.push(BaitAttraction::SpecificFish { name: "Salmon".to_string(), bias: BaitBias::Low, quality: AttractionQuality::Bad });
                 ui.close();
            }
        });
    }

    fn edit_bias(&self, ui: &mut egui::Ui, bias: &mut BaitBias, id_salt: usize) {
        egui::ComboBox::from_id_salt(format!("bias_{}", id_salt))
            .selected_text(format!("{:?}", bias))
            .show_ui(ui, |ui| {
                ui.selectable_value(bias, BaitBias::Low, "Low");
                ui.selectable_value(bias, BaitBias::Medium, "Medium");
                ui.selectable_value(bias, BaitBias::High, "High");
            });
    }

    fn edit_quality(&self, ui: &mut egui::Ui, quality: &mut AttractionQuality, id_salt: usize) {
         egui::ComboBox::from_id_salt(format!("qual_{}", id_salt))
            .selected_text(format!("{:?}", quality))
            .show_ui(ui, |ui| {
                ui.selectable_value(quality, AttractionQuality::Bad, "Bad");
                ui.selectable_value(quality, AttractionQuality::Good, "Good");
            });
    }
    
    fn edit_rarity(&self, ui: &mut egui::Ui, rarity: &mut FishRarity, id_salt: usize) {
         egui::ComboBox::from_id_salt(format!("rar_{}", id_salt))
            .selected_text(format!("{:?}", rarity))
            .show_ui(ui, |ui| {
                 ui.selectable_value(rarity, FishRarity::Common, "Common");
                 ui.selectable_value(rarity, FishRarity::Uncommon, "Uncommon");
                 ui.selectable_value(rarity, FishRarity::Rare, "Rare");
                 ui.selectable_value(rarity, FishRarity::Elusive, "Elusive");
                 ui.selectable_value(rarity, FishRarity::Legendary, "Legendary");
                 ui.selectable_value(rarity, FishRarity::Mythical, "Mythical");
            });
    }

    fn edit_category(&self, ui: &mut egui::Ui, cat: &mut FishCategory, id_salt: usize) {
         egui::ComboBox::from_id_salt(format!("cat_{}", id_salt))
            .selected_text(format!("{:?}", cat))
            .show_ui(ui, |ui| {
                ui.selectable_value(cat, FishCategory::BaitFish, "BaitFish");
                ui.selectable_value(cat, FishCategory::Schooling, "Schooling");
                ui.selectable_value(cat, FishCategory::Predatory, "Predatory");
                ui.selectable_value(cat, FishCategory::BottomFeeder, "BottomFeeder");
                ui.selectable_value(cat, FishCategory::Forager, "Forager");
                ui.selectable_value(cat, FishCategory::Ornamental, "Ornamental");
                 ui.selectable_value(cat, FishCategory::Apex, "Apex");
                ui.selectable_value(cat, FishCategory::Abyssal, "Abyssal");
                 ui.selectable_value(cat, FishCategory::Mythological, "Mythological");
            });
    }
}