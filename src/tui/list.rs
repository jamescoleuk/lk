use anyhow::Result;
use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::script::{self, Function, Script};

use super::state::App;

pub fn find(scripts: &[script::Script]) -> Result<Option<(Script, Function)>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::from(scripts);
    let res = find_loop(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn find_loop<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<Option<(Script, Function)>> {
    let last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Read input loop
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left => app.filtered_items.unselect(),
                        KeyCode::Down => app.filtered_items.next(),
                        KeyCode::Up => app.filtered_items.previous(),
                        KeyCode::Esc => return Ok(None),
                        KeyCode::Char(c) => {
                            app.update_search_term(c.to_string().as_str());
                        }
                        KeyCode::Delete => app.delete_search_term_char(),
                        KeyCode::Backspace => app.delete_search_term_char(),
                        KeyCode::Enter => {
                            let selected = app.get_selected();
                            match selected {
                                Some(selected) => return Ok(Some(selected.source.clone())),
                                None => return Ok(None),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// This allow makes it neater to compose the UI
#[allow(clippy::vec_init_then_push)]
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // The search bar on top, the other stuff below
    let all = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Percentage(5)].as_ref())
        .split(f.size());
    // The other stuff
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(all[1]);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .filtered_items
        .items
        .iter_mut()
        .map(|i| {
            let lines = vec![Line::from(i.name.as_str())];
            ListItem::new(lines).style(Style::default().fg(Color::White).bg(Color::Black))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Functions"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.filtered_items.state);
    let block = Block::new().borders(Borders::NONE);
    let para = Paragraph::new(format!("> {}", app.search_term.as_str()))
        .style(Style::new().white().on_black())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(para.clone().block(block), all[0]);

    // TODO: path to script, function name, function description, maybe the code.
    let selected = app.get_selected();
    if let Some(selected) = selected {
        // First we need to set up the text we're going to display
        let relative_path = selected.source.0.path();
        let _absolute_path = selected
            .source
            .0
            .absolute_path
            .to_string_lossy()
            .to_string();
        let mut file_comments: Vec<Line> = selected
            .source
            .0
            .comment
            .iter()
            .map(|c| Line::from(c.clone()))
            .collect();
        let _script_name = selected.source.0.file_name();
        let mut function_comments: Vec<Line> = selected
            .source
            .1
            .comment
            .iter()
            .map(|c| Line::from(c.clone()))
            .collect();

        let mut text = Vec::new();
        text.push(Line::from("Location".on_blue()));
        text.push(Line::from(relative_path));
        text.push(Line::from(""));
        text.push(Line::from("File comments".on_blue()));
        text.append(&mut file_comments);
        text.push(Line::from(""));
        text.push(Line::from("Function comments".on_blue()));
        text.append(&mut function_comments);

        // Finally we can create the paragraph and render it
        let para = Paragraph::new(text)
            .style(Style::new().white().on_black())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(
            para.clone().block(Block::new().borders(Borders::ALL)),
            chunks[1],
        );
    }
}
