mod data;
mod error;
mod ui;
mod event;


use anyhow::Result;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{thread, io};
use crossterm::event as CrosstermEvent;
use crossterm::event::{Event as CEvent, KeyCode};
use event::Event;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::layout::{Layout, Direction, Constraint, Alignment};
use tui::widgets::{Paragraph, Block, Borders, BorderType, Tabs, ListState};
use tui::style::{Style, Color, Modifier};
use ui::MenuItem;
use tui::text::{Spans, Span};
use crate::ui::{render_menu, render_pets};
use crate::data::{add_random_pet_to_db, remove_pet_at_index, read_db};

fn main() -> Result<()> {
    enable_raw_mode().expect("can run in raw mode");
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    // event
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if CrosstermEvent::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = CrosstermEvent::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }
            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // ui
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // state
    let menu_titles = vec!["Home", "Pets", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;

    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(0));
    loop {
        // rendering
        terminal.draw(|rect| {
            let size = rect.size();
            // footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                        .as_ref(),
                )
                .split(size);
            let copyright = Paragraph::new("pet-CLI 2021 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );
            rect.render_widget(copyright, chunks[2]);

            // header
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1); // Home -> [H,ome]
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();
            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));
            rect.render_widget(tabs, chunks[0]);

            // 真ん中

            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_menu(), chunks[1]),
                MenuItem::Pets => {
                    let pets_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_pets(&pet_list_state);
                    rect.render_stateful_widget(left, pets_chunks[0], &mut pet_list_state);
                    rect.render_widget(right, pets_chunks[1]);
                }
            }
        });

        // receive input from the channel
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Pets,
                KeyCode::Char('a') => {
                    add_random_pet_to_db()?;
                }
                KeyCode::Char('d') => {
                    remove_pet_at_index(&mut pet_list_state)?;
                }
                KeyCode::Down => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db()?.len();
                        if selected >= amount_pets - 1 {
                            pet_list_state.select(Some(0));
                        } else {
                            pet_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db()?.len();
                        if selected > 0 {
                            pet_list_state.select(Some(selected - 1));
                        } else {
                            pet_list_state.select(Some(amount_pets - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }
    Ok(())
}
