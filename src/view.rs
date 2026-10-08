use std::time::Duration;

use eframe::egui::{self, Color32, Vec2};
use tokio::sync::{mpsc, watch};

const INPUTS_COUNT: usize = 24;
const OUTPUTS_COUNT: usize = 20;

#[derive(Clone)]
pub struct IoState {
    pub inputs: Vec<bool>,
    pub outputs: Vec<bool>,
    pub status: String,
}

impl IoState {
    pub fn new(inputs_count: usize, outputs_count: usize) -> Self {
        Self {
            inputs: vec![false; inputs_count],
            outputs: vec![false; outputs_count],
            status: "Connecting...".to_owned(),
        }
    }
}

pub enum OutputCommand {
    Set { address: usize },
    Reset { address: usize },
    Impulse { address: usize },
}

pub struct Window {
    state_rx: watch::Receiver<IoState>,
    command_tx: mpsc::Sender<OutputCommand>,
    state: IoState,
    command_input: String,
    messages: Vec<String>,
}

impl Window {
    pub fn new(
        state_rx: watch::Receiver<IoState>,
        command_tx: mpsc::Sender<OutputCommand>,
    ) -> Self {
        let state = state_rx.borrow().clone();
        Self {
            state_rx,
            command_tx,
            state,
            command_input: String::new(),
            messages: Vec::new(),
        }
    }

    fn execute_command(&mut self) {
        let command = self.command_input.trim().to_owned();
        self.command_input.clear();

        if command.is_empty() {
            return;
        }

        let Some((name, argument)) = command.split_once('(') else {
            self.push_message(format!("Invalid command: {command}"));
            return;
        };
        let Some(address) = argument.strip_suffix(')') else {
            self.push_message(format!("Invalid command: {command}"));
            return;
        };

        let Some(address) = parse_output_address(address) else {
            self.push_message(format!(
                "Invalid output address: {address}; use Q1..Q{OUTPUTS_COUNT}"
            ));
            return;
        };

        let output_command = match name.trim().to_ascii_lowercase().as_str() {
            "set" => OutputCommand::Set { address },
            "reset" => OutputCommand::Reset { address },
            "impulse" => OutputCommand::Impulse { address },
            _ => {
                self.push_message(format!("Unknown command: {name}"));
                return;
            }
        };

        match self.command_tx.try_send(output_command) {
            Ok(()) => self.push_message(format!("Queued: {command}")),
            Err(error) => self.push_message(format!("Could not send command: {error}")),
        }
    }

    fn push_message(&mut self, message: String) {
        const MAX_MESSAGES: usize = 50;
        if self.messages.len() == MAX_MESSAGES {
            self.messages.remove(0);
        }
        self.messages.push(message);
    }

    fn draw_signals(ui: &mut egui::Ui, id: &str, title: &str, values: &[bool]) {
        ui.heading(title);
        egui::Grid::new(id)
            .num_columns(INPUTS_COUNT)
            .spacing(Vec2::new(6.0, 6.0))
            .show(ui, |ui| {
                for (index, &value) in values.iter().enumerate() {
                    ui.vertical(|ui| {
                        ui.label(format!("{}", index + 1));
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(20.0), egui::Sense::hover());
                        let color = if value {
                            Color32::GREEN
                        } else {
                            Color32::DARK_GRAY
                        };
                        ui.painter().rect_filled(rect, 3.0, color);
                    });
                }
            });
    }
}

fn parse_output_address(address: &str) -> Option<usize> {
    let number = address
        .trim()
        .parse::<usize>()
        .ok()?;

    (1..=OUTPUTS_COUNT).contains(&number).then(|| number - 1)
}

impl eframe::App for Window {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.state = self.state_rx.borrow_and_update().clone();
        ui.ctx().request_repaint_after(Duration::from_millis(16));

        Self::draw_signals(ui, "inputs", "Inputs", &self.state.inputs);
        ui.separator();
        Self::draw_signals(ui, "outputs", "Outputs", &self.state.outputs);
        ui.separator();

        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.command_input)
                    .hint_text("Enter command...")
                    .desired_width(f32::INFINITY),
            );
            let enter_pressed =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            let send_clicked = ui.button("Send").clicked();

            if enter_pressed || send_clicked {
                self.execute_command();
            }
        });

        egui::ScrollArea::vertical()
            .max_height(120.0)
            .show(ui, |ui| {
                for message in &self.messages {
                    ui.label(message);
                }
            });
    }
}
