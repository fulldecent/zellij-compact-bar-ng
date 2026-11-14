use zellij_tile::prelude::InputMode;

use plugin::{render_line, visible_width, BarState, TabSpec, Theme};

fn sample(n: usize, active: usize) -> Vec<TabSpec> {
    (0..n)
        .map(|i| TabSpec {
            index: (i + 1) as u32,
            name: format!("Tab #{}", i + 1),
            active: i == active,
            fullscreen: false,
            sync: false,
        })
        .collect()
}

#[test]
fn compact_row_fills_every_width_1_to_100() {
    for mode in [InputMode::Normal, InputMode::Locked, InputMode::Tab] {
        for n in 1..=4 {
            let tabs = sample(n, n / 2);
            let state = BarState {
                tabs: &tabs,
                session_name: Some("demo"),
                hide_session_name: false,
                mode,
                dimmed: false,
                theme: Theme::default(),
                hover_col: None,
            };
            for cols in 1..=100 {
                let line = render_line(cols, &state);
                assert_eq!(
                    visible_width(&line),
                    cols,
                    "{mode:?} with {n} tabs at {cols}"
                );
            }
        }
    }
}
