use std::time::Instant;

use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    app::preset::Preset,
    tui::{
        command::input::InputCommand,
        state::{State, form::FormFocus, screen::Screen},
    },
};

impl State {
    /// Runs a input command and updates the state.
    pub(crate) fn run_input_command(&mut self, command: InputCommand) {
        match command {
            InputCommand::Handle(event) => {
                self.input.handle_event(&event);
                match self.form_focus {
                    FormFocus::Owner => self.repo.owner = self.input.value().to_string(),
                    FormFocus::Name => self.repo.name = self.input.value().to_string(),
                    FormFocus::Desc => self.repo.desc = self.input.value().to_string(),
                    FormFocus::Options | FormFocus::Preset => {}
                }
                if self.form_focus == FormFocus::Owner
                    || self.form_focus == FormFocus::Name
                    || self.form_focus == FormFocus::Desc
                {
                    self.last_blink = Instant::now();
                }
            }
            InputCommand::Enter => {
                if self.form_focus == FormFocus::Preset {
                    if let Some(preset) = Preset::ALL.get(self.preset_cursor) {
                        self.repo.apply_preset(*preset);
                    }
                } else if self.form_focus == FormFocus::Options
                    && let Some(id) = self.option_list.selected()
                {
                    if let Some(opt) = self.repo.options.get_mut(id) {
                        opt.checked = !opt.checked;
                    }
                } else {
                    let value = match self.form_focus {
                        FormFocus::Owner => self.repo.owner.clone(),
                        FormFocus::Name => self.repo.name.clone(),
                        FormFocus::Desc => self.repo.desc.clone(),
                        FormFocus::Options | FormFocus::Preset => return,
                    };
                    self.input = Input::new(value);
                    self.screen_mode = Screen::Editing;
                }
            }
            InputCommand::Confirm => {
                self.screen_mode = Screen::Form;
            }
            InputCommand::Exit => {
                self.input = Input::default();
                self.screen_mode = Screen::Form;
            }
        }
    }
}
