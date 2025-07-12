use eframe::egui::{Button, ScrollArea, Slider, TextEdit, Ui};

use crate::{DEFAULT_CONTROL_MODE, UiView, ViewEvent, ViewRequest};
use protocol::{AutoTuneCommand, ControlMode};

#[derive(Default)]
pub(super) struct CommandWindow {
    curr_control_mode: ControlMode,
    request: Option<ViewRequest>,
    // velocity command, unit: rpm
    curr_vel_cmd: f32,
    prev_vel_cmd: f32,
    // position command format: '(dist, vel, vel_end);'
    // Input data should be enclosed by parenthesis, and use ';' to indicate the
    // end of one command block, and the unit of each data is as follows:
    // 1. dist: rad
    // 2. vel: rpm
    // 3. vel_end: rpm, the end velocity of position command block, it is optional.
    //    If it is not given, the end velocity will be treated as 0
    pos_cmd: String,
    // auto tune command
    auto_tune_cmd: AutoTuneCommand,
}

impl CommandWindow {
    pub fn new() -> Self {
        Self {
            curr_control_mode: DEFAULT_CONTROL_MODE,
            ..Default::default()
        }
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().max_height(64.0).show(ui, |ui| {
            ui.add_sized(ui.available_size(), TextEdit::multiline(&mut self.pos_cmd));
        });

        let send_button = Button::new("Send");
        if ui
            .add_enabled(!self.pos_cmd.is_empty(), send_button)
            .clicked()
        {
            self.request = Some(ViewRequest::PositionControl(self.pos_cmd.clone()));
        }
    }

    fn display_velocity_command_panel(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.curr_vel_cmd, -3000.0..=3000.0).text("motor velocity cmd (rpm)"),
        );

        if self.curr_vel_cmd != self.prev_vel_cmd {
            self.prev_vel_cmd = self.curr_vel_cmd;
            self.request = Some(ViewRequest::VelocityControl(self.curr_vel_cmd));
        }
    }

    fn display_autotune_command_panel(&mut self, ui: &mut Ui) {
        ui.columns(2, |columns| {
            columns[0].add(
                Slider::new(&mut self.auto_tune_cmd.set_point, -3000.0..=3000.0)
                    .text("set point velocity (rpm)"),
            );
            columns[1].add(
                Slider::new(&mut self.auto_tune_cmd.output_limit, 0.0..=0.75).text("output limit"),
            );
        });

        let text = if !self.auto_tune_cmd.start {
            "start"
        } else {
            "stop"
        };
        let button = Button::new(text);
        if ui.add(button).clicked() {
            self.auto_tune_cmd.start = !self.auto_tune_cmd.start;
            self.request = Some(ViewRequest::AutoTuneControl(self.auto_tune_cmd.clone()));
        }
    }
}

impl UiView for CommandWindow {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Command setup");
        match self.curr_control_mode {
            ControlMode::Position => self.display_position_command_panel(ui),
            ControlMode::Velocity => self.display_velocity_command_panel(ui),
            ControlMode::Pid => self.display_autotune_command_panel(ui),
            _ => (),
        }
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        self.request.take()
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ControlModeUpdate((ok, mode)) => {
                if mode == ControlMode::StandStill {
                    self.reset();
                }

                if ok {
                    self.curr_control_mode = mode;
                }
            }
            ViewEvent::ProfileDataUpdate(data) => {
                // Turn off auto tune command when the motor is not moving.
                // The target board runs motion task every 5ms, and 30 rpm is the minimum speed
                // that can be detected by the encoder and this cycle time. The calculation is
                // as follows:
                // the 24H BLDC motor generates 400 pulses per rev, so:
                // 30 rpm = 0.5 rev/s
                // 0.5 rev/s * 400 pulses/rev = 200 pulses/s
                // 200 pulses/s / 200 Hz = 1 pulse per cycle
                // Here, I slightly increase the threshold to 40 rpm to prevent unstable behavior
                if self.auto_tune_cmd.start && data.act_vel.abs() <= 40.0 {
                    self.auto_tune_cmd.start = false;
                }
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.request = None;
        self.curr_vel_cmd = 0.0;
        self.prev_vel_cmd = 0.0;
        self.pos_cmd.clear();
        self.auto_tune_cmd.start = false;
    }
}
