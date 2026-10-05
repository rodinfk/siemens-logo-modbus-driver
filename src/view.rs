use eframe::egui::{self, Color32, Vec2};

use std::sync::mpsc::{self, Receiver};
use crate::Changes;




struct Window {
    inputs: Vec<bool>,
    outputs: Vec<bool>,

    rx: Receiver<Changes>,

    command_input: String,
    output: Vec<String>,
}

impl Window {
    fn set(&mut self, addr: &str) {
        let message = format!("set({addr})");
        self.output.push(message);
    }

    fn reset(&mut self, addr: &str) {
        let message = format!("reset({addr})");
        self.output.push(message);
    }

    fn impulse(&mut self, addr: &str) {
        let message = format!("impulse({addr})");
        self.output.push(message);
    }

    fn execute_command(&mut self) {
        let command = self.command_input.trim().to_string();

        if command.is_empty() {
            return;
        }

        if let Some(addr) = command
            .strip_prefix("set(")
            .and_then(|s| s.strip_suffix(')'))
        {
            self.set(addr);
        } else if let Some(addr) = command
            .strip_prefix("reset(")
            .and_then(|s| s.strip_suffix(')'))
        {
            self.reset(addr);
        } else if let Some(addr) = command
            .strip_prefix("impulse(")
            .and_then(|s| s.strip_suffix(')'))
        {
            self.impulse(addr);
        } else {
            self.output.push(format!("Unknown command: {command}"));
        }

        self.command_input.clear();
    }
}

impl eframe::App for Window {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let columns = 24;
        let cell_size = Vec2::splat(20.0);

        egui::Grid::new("bool_grid")
            .spacing(Vec2::new(5.0, 5.0))
            .show(ui, |ui| {
                for (i, &value) in self.inputs.iter().enumerate() {
                    let color = if value {
                        Color32::GREEN
                    } else {
                        Color32::GRAY
                    };

                    ui.vertical(|ui| {
                        ui.set_width(cell_size.x);

                        ui.horizontal(|ui| {
                            ui.label(i.to_string());
                        });

                        let (rect, _) = ui.allocate_exact_size(
                            cell_size,
                            egui::Sense::hover(),
                        );

                        ui.painter().rect_filled(
                            rect,
                            3.0,
                            color,
                        );
                    });

                    if (i + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });

        ui.separator();

        let response = ui.add(
            egui::TextEdit::singleline(&mut self.command_input)
                .hint_text("Enter command...")
                .desired_width(f32::INFINITY),
        );

        if response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
        {
            self.execute_command();
        }
    }
}