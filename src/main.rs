use std::collections::BTreeMap;

use zellij_tile::prelude::actions::{Action, SearchDirection, SearchOption};
use zellij_tile::prelude::*;
use zellij_utils::input::layout::{PluginAlias, RunPluginOrAlias};

use plugin::{render_hits, render_line, Action as BarAction, BarHit, BarState, TabAction, TabSpec};

struct State {
    tabs: Vec<TabSpec>,
    session_name: Option<String>,
    hide_session_name: bool,
    mode: InputMode,
    dimmed: bool,
    theme: plugin::Theme,
    hover_col: Option<usize>,
    hits: Vec<BarHit>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            session_name: None,
            hide_session_name: false,
            mode: InputMode::Normal,
            dimmed: false,
            theme: plugin::Theme::default(),
            hover_col: None,
            hits: Vec::new(),
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::RunActionsAsUser,
        ]);
        subscribe(&[
            EventType::ModeUpdate,
            EventType::TabUpdate,
            EventType::Mouse,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(status) => {
                if status == PermissionStatus::Granted {
                    // Chrome, not a pane. Stay selectable until this event so
                    // the y/n permission prompt can receive a key.
                    set_selectable(false);
                }
                true
            }
            Event::ModeUpdate(info) => {
                self.mode = info.mode;
                self.session_name = info.session_name;
                self.hide_session_name = info.style.hide_session_name;
                self.dimmed =
                    info.session_dimmed == Some(true) || info.session_ascended == Some(true);
                self.theme = plugin::chrome_from(&info.style);
                true
            }
            Event::TabUpdate(tabs) => {
                self.tabs = tabs
                    .into_iter()
                    .map(|tab| TabSpec {
                        index: (tab.position + 1) as u32,
                        name: tab.name,
                        active: tab.active,
                        fullscreen: tab.is_fullscreen_active,
                        sync: tab.is_sync_panes_active,
                    })
                    .collect();
                true
            }
            Event::Mouse(Mouse::Hover(_, col)) => {
                if self.hover_col == Some(col) {
                    false
                } else {
                    self.hover_col = Some(col);
                    true
                }
            }
            Event::Mouse(Mouse::LeftClick(_, col)) => self.press(col),
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let state = BarState {
            tabs: &self.tabs,
            session_name: self.session_name.as_deref(),
            hide_session_name: self.hide_session_name,
            mode: self.mode,
            dimmed: self.dimmed,
            theme: self.theme,
            hover_col: self.hover_col,
        };
        self.hits = render_hits(cols, &state);
        print!("{}", render_line(cols, &state));
    }
}

impl State {
    fn press(&mut self, col: usize) -> bool {
        let Some(hit) = self
            .hits
            .iter()
            .find(|hit| col >= hit.start && col < hit.end)
        else {
            return false;
        };
        match hit.action {
            BarAction::Tab(TabAction::Switch(index) | TabAction::Reveal(index)) => {
                switch_tab_to(index);
            }
            BarAction::Tab(TabAction::NewTab) => {
                new_tab::<&str>(None, None);
            }
            BarAction::Status(click) => run_click(click),
        }
        false
    }
}

fn input_mode(name: &str) -> Option<InputMode> {
    Some(match name {
        "normal" => InputMode::Normal,
        "locked" => InputMode::Locked,
        "pane" => InputMode::Pane,
        "tab" => InputMode::Tab,
        "resize" => InputMode::Resize,
        "move" => InputMode::Move,
        "scroll" => InputMode::Scroll,
        "enter_search" => InputMode::EnterSearch,
        "search" => InputMode::Search,
        "rename_tab" => InputMode::RenameTab,
        "rename_pane" => InputMode::RenamePane,
        "session" => InputMode::Session,
        "tmux" => InputMode::Tmux,
        _ => return None,
    })
}

fn launch(url: &str) {
    let plugin = RunPluginOrAlias::from_url(url, &None, None, None)
        .unwrap_or_else(|_| RunPluginOrAlias::Alias(PluginAlias::new(url, &None, None)));
    run_action(
        Action::LaunchOrFocusPlugin {
            plugin,
            should_float: true,
            move_to_focused_tab: true,
            should_open_in_place: false,
            close_replaced_pane: false,
            skip_cache: false,
            tab_id: None,
        },
        BTreeMap::new(),
    );
    switch_to_input_mode(&InputMode::Normal);
}

fn run_click(click: plugin::Click) {
    match click {
        plugin::Click::Mode(name) => {
            if let Some(mode) = input_mode(name) {
                switch_to_input_mode(&mode);
            }
        }
        plugin::Click::Quit => quit_zellij(),
        plugin::Click::NewPane => {
            new_pane();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::NewPaneDown => {
            run_action(
                Action::NewTiledPane {
                    direction: Some(Direction::Down),
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    borderless: None,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::NewPaneRight => {
            run_action(
                Action::NewTiledPane {
                    direction: Some(Direction::Right),
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    borderless: None,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::NewStacked => {
            run_action(
                Action::NewStackedPane {
                    command: None,
                    pane_name: None,
                    near_current_pane: false,
                    no_focus: false,
                    tab_id: None,
                },
                BTreeMap::new(),
            );
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::FocusLeft => move_focus(Direction::Left),
        plugin::Click::FocusDown => move_focus(Direction::Down),
        plugin::Click::FocusUp => move_focus(Direction::Up),
        plugin::Click::FocusRight => move_focus(Direction::Right),
        plugin::Click::ResizeIncrease => resize_focused_pane(Resize::Increase),
        plugin::Click::ResizeDecrease => resize_focused_pane(Resize::Decrease),
        plugin::Click::ResizeIncreaseLeft => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Left)
        }
        plugin::Click::ResizeIncreaseDown => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Down)
        }
        plugin::Click::ResizeIncreaseUp => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Up)
        }
        plugin::Click::ResizeIncreaseRight => {
            resize_focused_pane_with_direction(Resize::Increase, Direction::Right)
        }
        plugin::Click::ResizeDecreaseLeft => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Left)
        }
        plugin::Click::ResizeDecreaseDown => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Down)
        }
        plugin::Click::ResizeDecreaseUp => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Up)
        }
        plugin::Click::ResizeDecreaseRight => {
            resize_focused_pane_with_direction(Resize::Decrease, Direction::Right)
        }
        plugin::Click::MovePaneLeft => move_pane_with_direction(Direction::Left),
        plugin::Click::MovePaneDown => move_pane_with_direction(Direction::Down),
        plugin::Click::MovePaneUp => move_pane_with_direction(Direction::Up),
        plugin::Click::MovePaneRight => move_pane_with_direction(Direction::Right),
        plugin::Click::ToggleFloating => {
            toggle_floating_panes(None);
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::ToggleEmbed => {
            toggle_pane_embed_or_eject();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::ToggleFullscreen => {
            toggle_focus_fullscreen();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::ClosePane => {
            close_focus();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::CloseTab => {
            close_focused_tab();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::NewTab => {
            new_tab::<&str>(None, None);
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::RenamePane => switch_to_input_mode(&InputMode::RenamePane),
        plugin::Click::RenameTab => switch_to_input_mode(&InputMode::RenameTab),
        plugin::Click::SyncTab => {
            toggle_active_tab_sync();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::BreakPane => {
            run_action(Action::BreakPane, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::BreakPaneLeft => {
            run_action(Action::BreakPaneLeft, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::BreakPaneRight => {
            run_action(Action::BreakPaneRight, BTreeMap::new());
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::ToggleTab => toggle_tab(),
        plugin::Click::SelectPane => switch_to_input_mode(&InputMode::Normal),
        plugin::Click::Detach => detach(),
        plugin::Click::SessionManager => launch("session-manager"),
        plugin::Click::Configuration => launch("configuration"),
        plugin::Click::PluginManager => launch("plugin-manager"),
        plugin::Click::About => launch("zellij:about"),
        plugin::Click::Share => launch("zellij:share"),
        plugin::Click::LayoutManager => launch("zellij:layout-manager"),
        plugin::Click::EditScrollback => {
            edit_scrollback();
            switch_to_input_mode(&InputMode::Normal);
        }
        plugin::Click::ScrollUp => scroll_up(),
        plugin::Click::ScrollDown => scroll_down(),
        plugin::Click::PageScrollUp => page_scroll_up(),
        plugin::Click::PageScrollDown => page_scroll_down(),
        plugin::Click::EnterSearch => switch_to_input_mode(&InputMode::EnterSearch),
        plugin::Click::CancelSearch => switch_to_input_mode(&InputMode::Scroll),
        plugin::Click::ConfirmSearch => switch_to_input_mode(&InputMode::Search),
        plugin::Click::SearchDown => run_action(
            Action::Search {
                direction: SearchDirection::Down,
            },
            BTreeMap::new(),
        ),
        plugin::Click::SearchUp => run_action(
            Action::Search {
                direction: SearchDirection::Up,
            },
            BTreeMap::new(),
        ),
        plugin::Click::SearchCase => run_action(
            Action::SearchToggleOption {
                option: SearchOption::CaseSensitivity,
            },
            BTreeMap::new(),
        ),
        plugin::Click::SearchWrap => run_action(
            Action::SearchToggleOption {
                option: SearchOption::Wrap,
            },
            BTreeMap::new(),
        ),
        plugin::Click::SearchWhole => run_action(
            Action::SearchToggleOption {
                option: SearchOption::WholeWord,
            },
            BTreeMap::new(),
        ),
        plugin::Click::TabPrev => go_to_previous_tab(),
        plugin::Click::TabNext => go_to_next_tab(),
        plugin::Click::DoneRenamePane => {
            undo_rename_pane();
            switch_to_input_mode(&InputMode::Pane);
        }
        plugin::Click::DoneRenameTab => {
            undo_rename_tab();
            switch_to_input_mode(&InputMode::Tab);
        }
    }
}
