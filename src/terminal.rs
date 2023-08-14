use anyhow::{Error, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::Stdout;
use std::time::Duration;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let stdout = std::io::stdout();
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let term = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(8),
        },
    )?;
    Ok(term)
}

/// Interactively select one item from a list
pub fn run_list_selection<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    items: Vec<(&'a String, &'a String)>,
) -> Result<(&'a String, &'a String)> {
    let mut list = StatefulList::with_items(items);
    list.state.select(Some(0));

    loop {
        let items: Vec<ListItem> = list
            .items
            .iter()
            .map(|(_, v)| ListItem::new(v.to_string()))
            .collect();

        let widget = List::new(items)
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::ITALIC)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(100)])
                .split(f.size());
            f.render_stateful_widget(widget, chunks[0], &mut list.state);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Err(Error::msg("Nothing selected"));
                        }
                        KeyCode::Enter => {
                            let i = list
                                .state
                                .selected()
                                .ok_or(Error::msg("Nothing selected"))?;
                            let selected = list
                                .items
                                .get(i)
                                .ok_or(Error::msg("Invalid selection"))?
                                .to_owned();
                            return Ok(selected);
                        }
                        KeyCode::Left => list.unselect(),
                        KeyCode::Down => list.next(),
                        KeyCode::Up => list.previous(),
                        KeyCode::Char('c') => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                return Err(Error::msg("Nothing selected"));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub fn restore_terminal<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.clear()?;
    disable_raw_mode()?;
    Ok(())
}
