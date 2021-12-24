use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdout, Stdout, Write};
use std::time::Instant;
use termion::color;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

use crate::fuzzy::get_coloured_line;

use super::item::Item;
use super::list::List;

pub struct FuzzyFinder<T>
where
    T: Clone,
{
    search_term: String,
    all_items: Vec<Item<T>>,
    matches: Vec<Item<T>>,
    console_offset: u16,
    stdout: RawTerminal<Stdout>,
    first: bool,
    view: List<T>,
    positive_space_remaining: u16,
}

impl<T> FuzzyFinder<T>
where
    T: Clone,
{
    fn new(functions: Vec<Item<T>>) -> Self {
        // We need to know where to start rendering from. We can't do this later because
        // we overwrite the cursor. Maybe we shouldn't do this? (TODO)
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "{}", termion::cursor::Save).unwrap();
        let lines_to_show: i8 = 8;
        let mut positive_space_remaining = 0;
        let console_offset = if stdout.cursor_pos().is_ok() {
            let cursor_pos_y = stdout.cursor_pos().unwrap().1;

            let terminal_height = termion::terminal_size().unwrap().1;
            let starting_y = cursor_pos_y;
            let ending_y = starting_y + lines_to_show as u16;
            let space_remaining: i16 = terminal_height as i16 - ending_y as i16;
            positive_space_remaining = if space_remaining < 0 {
                space_remaining.abs().try_into().unwrap()
            } else {
                0
            };
            cursor_pos_y
        } else {
            log::error!("Cannot get cursor!");
            0
        };

        FuzzyFinder {
            search_term: String::from(""),
            all_items: functions,
            matches: vec![],
            console_offset,
            stdout,
            first: true,
            view: List::new(lines_to_show),
            positive_space_remaining,
        }
    }

    pub fn up(&mut self) -> Result<()> {
        self.view.up(&self.matches);
        self.update_matches();
        self.render()
    }

    pub fn down(&mut self) -> Result<()> {
        self.view.down();
        self.update_matches();
        self.render()
    }

    pub fn append(&mut self, c: char) -> Result<()> {
        // This is a normal key that we want to add to the search.
        self.search_term = format!("{}{}", self.search_term, c);

        self.update_matches();
        self.render()
    }

    pub fn backspace(&mut self) -> Result<()> {
        if self.search_term.chars().count() > 0 {
            self.search_term =
                String::from(&self.search_term[..self.search_term.chars().count() - 1]);
            let matcher = SkimMatcherV2::default();
            for f in &mut self.all_items {
                f.score = matcher.fuzzy_indices(&f.name, &self.search_term);
            }
        }
        self.update_matches();
        self.render()
    }

    fn render_space(&mut self) -> Result<()> {
        // Drop down so we don't over-write the terminal line that instigated
        // this run of lk.
        write!(self.stdout, "{}", termion::cursor::Save).unwrap();
        if self.first {
            for _ in 0..self.view.lines_to_show {
                writeln!(self.stdout, " ")?;
            }
            self.first = false
        }
        write!(self.stdout, "{}", termion::cursor::Restore).unwrap();

        Ok(())
    }

    fn goto_start(&mut self) -> Result<()> {
        write!(
            self.stdout,
            "{}",
            termion::cursor::Goto(1, self.console_offset - self.positive_space_remaining)
        )?;
        Ok(())
    }

    fn render_items(&mut self) -> Result<()> {
        self.goto_start()?;
        for (index, item) in self.view.contents.iter().enumerate() {
            if item.is_blank {
                writeln!(self.stdout, "{}", termion::clear::CurrentLine)?;
            } else {
                let fuzzy_indecies = &item.score.as_ref().unwrap().1;

                // Do some string manipulation to colourise the indexed parts
                let coloured_line = get_coloured_line(
                    fuzzy_indecies,
                    &item.name,
                    index == self.view.selected_index as usize,
                );

                writeln!(
                    self.stdout,
                    "{}{}{}",
                    termion::clear::CurrentLine,
                    // Go maximum left, so we're at the start of the line
                    termion::cursor::Left(1000),
                    coloured_line
                )?;
            }
        }
        Ok(())
    }

    fn render_prompt(&mut self) -> Result<()> {
        // Render the prompt
        let prompt_y = self.view.lines_to_show as u16 + 1;
        let current_x = self.search_term.chars().count() + 2;

        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::CurrentLine,
            termion::cursor::Goto(current_x as u16, prompt_y + self.console_offset),
            termion::clear::CurrentLine,
        )?;
        write!(
            self.stdout,
            "{}{}{}${} {}",
            termion::cursor::Show,
            termion::cursor::Goto(1, prompt_y + self.console_offset),
            color::Fg(color::Cyan),
            color::Fg(color::Reset),
            self.search_term
        )?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Gets functions that match our current criteria, sorted by score.
    pub fn update_matches(&mut self) {
        let matcher = SkimMatcherV2::default();
        for f in &mut self.all_items {
            f.score = matcher.fuzzy_indices(&f.name, &self.search_term);
        }
        let mut matches = self
            .all_items
            .iter()
            .filter(|f| f.score.is_some())
            .cloned()
            .collect::<Vec<Item<T>>>();

        log::info!(
            "There are a total of {} item(s) and {} match(es)",
            self.all_items.len(),
            matches.len()
        );

        // We want these in the order of their fuzzy matched score, i.e. closed matches
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        self.matches = matches;
        self.view.update(&self.matches);
    }

    /// Renders the current result set
    pub fn render(&mut self) -> Result<()> {
        self.render_space()?;
        self.render_items()?;
        self.render_prompt()?;
        Ok(())
    }

    /// The main entry point for the fuzzy finder.
    pub fn find(items: Vec<Item<T>>) -> Result<Option<T>> {
        let mut state = FuzzyFinder::new(items);

        state.update_matches();

        state.render()?;

        let mut stdin = termion::async_stdin().keys();

        // Run 'sed -n l' to explore escape codes
        let mut escaped = String::from("");
        let mut instant = Instant::now();

        loop {
            // What's going on here? The problem is how we detect escape.
            // The key presses we're interested in, e.g. the arrows, are all preceded by escape, ^[.
            // E.g. up is ^[[A and down is ^[[B. So the question is how do we identify an escape
            // key by itself? If it's ^[[A then that's ^[ followed almost instantly by [A. If we have
            // ^[ followed by a pause then we know it's not an escape for some other key, but an
            // escape by itself. That's what the 100 136His below.
            // NB: some terminals might send these bytes too slowly and escape might not be caught.
            // NB: some terminals might use different escape keys entirely.
            if escaped == "^[" && instant.elapsed().as_micros() > 100 {
                write!(state.stdout, "{}", termion::cursor::Restore)?;
                break;
            }

            if let Some(Ok(key)) = stdin.next() {
                match key {
                    // ctrl-c and ctrl-d are two ways to exit.
                    Key::Ctrl('c') => break,
                    Key::Ctrl('d') => break,

                    // NB: It'd be neat if we could use Key::Up and Key::Down but they don't
                    // work in raw mode. So we've got to deal with the escape codes manually.

                    // This captures the enter key
                    Key::Char('\n') => {
                        return if !state.matches.is_empty() {
                            // Tidy up the console lines we've been writing
                            for _ in state.console_offset
                                ..state.console_offset + state.view.lines_to_show as u16 + 4
                            {
                                write!(state.stdout, "{}", termion::clear::CurrentLine,)?;
                            }
                            Ok(Some(
                                state.view.get_selected().item.as_ref().unwrap().to_owned(),
                            ))
                        } else {
                            Ok(None)
                        };
                    }
                    Key::Char(c) => {
                        if !escaped.is_empty() {
                            escaped = format!("{}{}", escaped, c);
                            match escaped.as_str() {
                                "^[" => continue,
                                "^[[" => continue,
                                "^[[A" => {
                                    escaped = String::from("");
                                    state.up()?;
                                }
                                "^[[B" => {
                                    escaped = String::from("");
                                    state.down()?;
                                }
                                _ => {
                                    // This is nothing we recognise so let's abandon the escape sequence.
                                    escaped = String::from("");
                                }
                            }
                        } else {
                            state.append(c)?;
                        }
                    }
                    Key::Esc => {
                        // All we're doing here is recording that we've entered an escape sequence.
                        // It's actually handled when we handle chars.
                        if escaped.is_empty() {
                            escaped = String::from("^[");
                            instant = Instant::now();
                        }
                    }
                    Key::Backspace => {
                        state.backspace()?;
                    }
                    _ => {}
                }
                state.stdout.flush().unwrap();
            }
        }
        Ok(None)
    }
}
