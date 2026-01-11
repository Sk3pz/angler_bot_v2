use eframe::egui;
use crate::data_management::config::Config;

pub struct ConfigEditor {
    config: Config,
    status_message: Option<(String, std::time::Instant)>,
}

impl ConfigEditor {
    pub fn new() -> Self {
        Self {
            config: Config::load(),
            status_message: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration Editor");

        if ui.button("Refresh / Reload from Disk").clicked() {
             self.config = Config::load();
        }

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // General
            ui.collapsing("General", |ui| {
                ui.horizontal(|ui| {
                    ui.label("MOTD:");
                    ui.text_edit_singleline(&mut self.config.general.motd);
                });
                ui.checkbox(&mut self.config.general.log_cast_data, "Log Cast Data");
            });

            // Fishing
            ui.collapsing("Fishing", |ui| {
                ui.add(egui::Slider::new(&mut self.config.fishing.fish_weight_time_multiplier, 0.1..=5.0).text("Weight Time Multiplier"));
                ui.add(egui::Slider::new(&mut self.config.fishing.base_catch_chance, 0.0..=1.0).text("Base Catch Chance"));
                ui.add(egui::Slider::new(&mut self.config.fishing.base_cast_wait, 1.0..=60.0).text("Base Cast Wait"));
                
                ui.horizontal(|ui| {
                    ui.label("Min Cast Wait:");
                    ui.add(egui::DragValue::new(&mut self.config.fishing.min_cast_wait));
                });
                 ui.horizontal(|ui| {
                    ui.label("Max Cast Wait:");
                    ui.add(egui::DragValue::new(&mut self.config.fishing.max_cast_wait));
                });
                
                ui.add(egui::Slider::new(&mut self.config.fishing.base_qte_time, 1.0..=60.0).text("Base QTE Time"));
            });

             // Baoit
            ui.collapsing("Bait", |ui| {
                ui.add(egui::Slider::new(&mut self.config.bait.low_bait_weight, 0.0..=10.0).text("Low Bait Weight"));
                ui.add(egui::Slider::new(&mut self.config.bait.medium_bait_weight, 0.0..=10.0).text("Medium Bait Weight"));
                 ui.add(egui::Slider::new(&mut self.config.bait.high_bait_weight, 0.0..=10.0).text("High Bait Weight"));
            });
        });
        
        ui.separator();
        
        if ui.button("Save Config").clicked() {
            self.config.save();
            self.status_message = Some(("Scanner Config saved!".to_string(), std::time::Instant::now()));
        }

        if let Some((msg, time)) = &self.status_message {
            if time.elapsed().as_secs() < 3 {
                ui.label(msg);
            }
        }
    }
}
