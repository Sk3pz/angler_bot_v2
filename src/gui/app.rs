use eframe::egui;
use serenity::http::Http;
use std::sync::Arc;

use crate::gui::{
    console::Console,
    config_editor::ConfigEditor,
    user_editor::UserEditor,
};

pub struct AnglerApp {
    console: Console,
    config_editor: ConfigEditor,
    user_editor: UserEditor,
    selected_tab: Tab,
}

#[derive(PartialEq, Debug)]
enum Tab {
    Console,
    UserFile,
    Config,
}

impl AnglerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, http: Option<Arc<Http>>) -> Self {
        Self {
            console: Console::new(),
            config_editor: ConfigEditor::new(),
            user_editor: UserEditor::new(http),
            selected_tab: Tab::Console,
        }
    }
}

impl eframe::App for AnglerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, Tab::Console, "Console");
                ui.selectable_value(&mut self.selected_tab, Tab::UserFile, "User Files");
                ui.selectable_value(&mut self.selected_tab, Tab::Config, "Config");
            });

            ui.separator();

            match self.selected_tab {
                Tab::Console => self.console.show(ui),
                Tab::UserFile => self.user_editor.show(ui),
                Tab::Config => self.config_editor.show(ui),
            }
        });
        
        // Repaint constantly for console updates? Or trigger it elsewhere.
        // For console, it's good to request repaint if new logs came in.
        // But for now, let's just request repaint every second or let inputs drive it.
        // Actually, logs come from background threads, so we should request repaint.
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
