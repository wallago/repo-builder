use std::time::{Duration, Instant};

use ratatui::{
    style::{Color, Style},
    widgets::ListState,
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use tui_input::Input;

use crate::{
    app::RepoBuilder,
    config::{Config, binds::Keybindings},
};
use crate::{app::preset::Preset, prelude::*};

/// Running command.
pub(crate) mod command;

/// Helper to get key bindings.
pub(crate) mod binds;

/// Toast.
mod toast;

/// Screen mode.
pub(crate) mod screen;

/// Form.
pub(crate) mod form;

/// Staged Panel.
pub(crate) mod staged_panel;

/// Application state.
#[derive(Debug)]
pub struct State {
    /// Is the application running?
    pub running: bool,
    /// Terminal accent color.
    pub accent_color: Color,
    /// Repo builder.
    pub repo: RepoBuilder,
    /// Form focus.
    pub form_focus: form::FormFocus,
    /// Input.
    pub input: Input,
    /// Is cursor is active.
    pub cursor_active: bool,
    /// Last blink.
    pub last_blink: Instant,
    /// Screen mode.
    pub screen_mode: screen::Screen,
    /// List of options.
    pub option_list: ListState,
    /// Active key bindings (defaults merged with `config.toml`).
    pub keybindings: Keybindings,
    /// Current toast, if any (non-tokio: we manage timing ourselves).
    pub toast: Option<toast::Toast>,
    /// When the current toast should be hidden.
    pub toast_expires_at: Option<Instant>,
    /// List of staged files.
    pub staged_list: ListState,
    /// Staged panel focus.
    pub staged_panel_focus: staged_panel::StagedPanelFocus,
    /// Scroll staged content viewport.
    pub staged_content_viewport: usize,
    /// File explorer.
    pub explorer: FileExplorer,
    /// Cursor into `Preset::ALL` for the preset row.
    pub preset_cursor: usize,
}

impl State {
    /// Constructs a new instance of [`State`].
    pub(crate) fn new(accent_color: Option<Color>, config: Config) -> Result<Self> {
        let repo = RepoBuilder::default();
        let theme = Theme::default()
            .with_highlight_symbol("> ")
            .with_highlight_item_style(
                Style::default()
                    .fg(accent_color.unwrap_or(Color::Gray))
                    .bold(),
            )
            .with_highlight_dir_style(
                Style::default()
                    .fg(accent_color.unwrap_or(Color::Gray))
                    .bold(),
            );
        // Open on whichever preset the default tool selection already matches.
        let preset_cursor = repo
            .active_preset()
            .and_then(|active| Preset::ALL.iter().position(|preset| *preset == active))
            .unwrap_or(0);
        Ok(Self {
            running: true,
            accent_color: accent_color.unwrap_or(Color::White),
            repo,
            form_focus: form::FormFocus::Owner,
            input: Input::default(),
            screen_mode: screen::Screen::Form,
            option_list: ListState::default().with_selected(Some(0)),
            keybindings: config.keybindings,
            last_blink: Instant::now(),
            cursor_active: false,
            toast: None,
            toast_expires_at: None,
            staged_list: ListState::default().with_selected(Some(0)),
            staged_panel_focus: staged_panel::StagedPanelFocus::List,
            staged_content_viewport: 0,
            explorer: FileExplorerBuilder::build_with_theme(theme)?,
            preset_cursor,
        })
    }

    /// Display toast on call.
    fn show_toast(&mut self, msg: &str, kind: toast::ToastType) {
        self.toast = Some(toast::Toast {
            message: msg.to_string(),
            kind,
        });
        self.toast_expires_at = Some(Instant::now() + Duration::from_secs(3));
    }

    /// Hide the current toast once its lifetime has elapsed. Must be called
    /// every loop iteration since we manage toast timing ourselves (non-tokio).
    pub(crate) fn tick_toasts(&mut self) {
        if let Some(expiry) = self.toast_expires_at
            && Instant::now() >= expiry
        {
            self.toast = None;
            self.toast_expires_at = None;
        }
    }
}
