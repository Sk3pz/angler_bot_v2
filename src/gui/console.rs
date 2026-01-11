use eframe::egui;
use crate::gui::logging::GLOBAL_LOG_BUFFER;
use better_term::Color;

pub struct Console {
    // buffer is accessed via GLOBAL_LOG_BUFFER usually, but we can store local state if needed
}

impl Console {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Console Output");
        
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            if let Some(buffer) = GLOBAL_LOG_BUFFER.get() {
                let logs = buffer.get_logs();
                for log in logs {
                    let color = match log.color {
                        Color::Black => egui::Color32::BLACK,
                        Color::Red => egui::Color32::RED,
                        Color::Green => egui::Color32::GREEN,
                        Color::Yellow => egui::Color32::YELLOW,
                        Color::Blue => egui::Color32::BLUE,
                        Color::Purple => egui::Color32::from_rgb(128, 0, 128),
                        Color::Cyan => egui::Color32::from_rgb(0, 255, 255),
                        Color::White => egui::Color32::WHITE,
                        Color::BrightBlack => egui::Color32::GRAY,
                        Color::BrightRed => egui::Color32::LIGHT_RED,
                        Color::BrightGreen => egui::Color32::LIGHT_GREEN,
                        Color::BrightYellow => egui::Color32::LIGHT_YELLOW,
                        Color::BrightBlue => egui::Color32::LIGHT_BLUE,
                        Color::BrightPurple => egui::Color32::from_rgb(255, 0, 255),
                        Color::BrightCyan => egui::Color32::from_rgb(224, 255, 255),
                        Color::BrightWhite => egui::Color32::WHITE,
                        _ => egui::Color32::GRAY,
                    };
                    
                    ui.colored_label(color, &log.message);
                }
            } else {
                ui.label("Log buffer not initialized");
            }
        });
    }
}
