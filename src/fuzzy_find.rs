use crate::script::{Function, Script};
use anyhow::Result;
use crossterm::style::Stylize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdout, Stdout, Write};
use std::time::Instant;
use termion::color;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

struct View {
    top_index: u8,
    bottom_index: u8,
    lines_to_show: i8,
    selected_index: i8,
    contents: Option<Vec<Option<FuzzyFunction>>>,
}

impl View {
    pub fn new(lines_to_show: i8) -> Self {
        View {
            contents: Option::None,
            top_index: lines_to_show as u8 - 1,
            selected_index: (lines_to_show - 1) as i8,
            lines_to_show,
            bottom_index: 0,
        }
    }

    /// Takes the current matches and updates the visible contents.
    pub fn update(&mut self, matches: &[FuzzyFunction]) {
        let mut to_render: Vec<Option<FuzzyFunction>> = Vec::new();
        // Get everything in our display window
        for i in self.bottom_index..self.top_index + 1 {
            if matches.len() > (i).into() {
                to_render.push(Option::Some(matches[i as usize].clone()));
            } else {
                to_render.push(Option::None);
            }
        }

        // Now that the order is reversed our indexes will match. If the selected_index
        // is outside the range of what's selectable, i.e. our matches, then we need
        // to gently reset it back to the limit. This prevents the selection going onto
        // blank lines and also moves the selection to the top of the matches when
        // the number of matches shrinks.
        for (i, item) in to_render.iter().enumerate() {
            if item.is_none() {
                log::info!("selected_index: {}, i: {}", self.selected_index, i);
                self.selected_index = if self.selected_index <= i as i8 {
                    self.lines_to_show - matches.len() as i8
                } else {
                    self.selected_index
                }
            }
        }

        to_render.reverse();
        self.contents = Some(to_render);
    }
}

struct UiState {
    search_term: String,
    fuzzy_functions: Vec<FuzzyFunction>,
    matches: Option<Vec<FuzzyFunction>>,
    console_offset: u16,
    stdout: RawTerminal<Stdout>,
    first: bool,
    view: View,
}

impl UiState {
    pub fn new(functions: Vec<FuzzyFunction>) -> Self {
        // We need to know where to start rendering from. We can't do this later because
        // we overwrite the cursor. Maybe we shouldn't do this? (TODO)
        let mut stdout = stdout().into_raw_mode().unwrap();
        let lines_to_show: i8 = 8;
        let console_offset = if stdout.cursor_pos().is_ok() {
            stdout.cursor_pos().unwrap().1 - (&lines_to_show + 2) as u16
        } else {
            log::error!("Cannot get cursor!");
            0
        };

        let mut state = UiState {
            search_term: String::from(""),
            fuzzy_functions: functions,
            matches: Option::None,
            console_offset,
            stdout,
            first: true,
            view: View::new(lines_to_show),
        };
        state.update_matches();
        state
    }

    pub fn up(&mut self) -> Result<()> {
        log::info!("------------- up -------------");
        let match_count = self.matches.as_ref().unwrap().len() as i8;
        log::info!(
            "selected_index: {}, match_count: {}, bottom_index: {}, top_index: {}",
            self.view.selected_index,
            match_count,
            self.view.bottom_index,
            self.view.top_index
        );
        // if self.selected_index > 0 && self.selected_index < match_count {
        if self.view.selected_index > 0 {
            log::info!("{} - {}", self.view.selected_index, match_count);
            self.view.selected_index -= 1;
        } else if self.view.top_index < (match_count - 1) as u8 {
            log::info!("not going up because we're at the limit");
            self.view.bottom_index += 1;
            self.view.top_index += 1;
        }
        self.render()
    }

    pub fn down(&mut self) -> Result<()> {
        log::info!("------------- down -------------");
        let match_count = self.matches.as_ref().unwrap().len() as i8;
        log::info!(
            " selected_index: {}, match_count: {}, bottom_index: {}",
            self.view.selected_index,
            match_count,
            self.view.bottom_index,
        );
        // if self.selected_index < match_count - 1 && self.selected_index >= self.bottom_index as i8 {

        // Should we move the selection down?
        if self.view.selected_index < self.view.top_index as i8 {
            log::info!("incrementing");
            self.view.selected_index += 1;
        }

        // Should we scroll down?
        if self.view.selected_index > self.view.lines_to_show - 1 && self.view.bottom_index > 0 {
            self.view.bottom_index -= 1;
            self.view.top_index -= 1;
            // if we've scrolled down then we don't want to change the selected index
            // The selected index is for the view, so it stays the same.
            if self.view.selected_index > 0 {
                self.view.selected_index -= 1;
            }
        } else {
            log::info!("not scrolling down own because we're at the limit");
        }

        self.render()
    }

    pub fn append(&mut self, c: char) -> Result<()> {
        log::info!("append");
        // This is a normal key that we want to add to the search.
        self.search_term = format!("{}{}", self.search_term, c);

        let matcher = SkimMatcherV2::default();
        for f in &mut self.fuzzy_functions {
            f.score = matcher.fuzzy_indices(&f.long_name, &self.search_term);
        }
        self.update_matches();
        self.render()
    }

    pub fn backspace(&mut self) -> Result<()> {
        log::info!("backspace");
        if self.search_term.chars().count() > 0 {
            self.search_term =
                String::from(&self.search_term[..self.search_term.chars().count() - 1]);
            let matcher = SkimMatcherV2::default();
            for f in &mut self.fuzzy_functions {
                f.score = matcher.fuzzy_indices(&f.long_name, &self.search_term);
            }
        }
        self.update_matches();
        self.render()
    }

    pub fn get_selected(&self) -> FuzzyFunction {
        let view = &mut self.view.contents.as_ref().unwrap().to_owned();
        let index = self.view.selected_index as usize;
        let selected = view[index].as_ref().unwrap();
        selected.to_owned()
    }

    /// Gets functions that match our current criteria, sorted by score.
    fn update_matches(&mut self) {
        log::info!("------------- update_matches -------------");
        let mut matches = self
            .fuzzy_functions
            .iter()
            .filter(|f| f.score.is_some())
            .cloned()
            .collect::<Vec<FuzzyFunction>>();

        // We want these in the order of their fuzzy matched score, i.e. closed matches
        // matches.sort_by(|a, b| a.score.cmp(&b.score));
        matches.sort_by(|a, b| b.score.cmp(&a.score));
        let match_count = matches.len() as i8;
        self.matches = Some(matches);

        // We can't have the index greater than the match count
        if self.view.selected_index >= match_count {
            self.view.selected_index = match_count - 1;
            log::info!("resetting selected_index against match count");
        }
    }

    fn get_coloured_line(
        fuzzy_indecies: &[usize],
        matching: &FuzzyFunction,
        is_selected: bool,
    ) -> String {
        // Do some string manipulation to colourise the indexed parts
        let mut coloured_line = String::from("");
        let mut start = 0;
        for i in fuzzy_indecies {
            let part = &matching.long_name[start..*i];
            let matching_char = &matching.long_name[*i..*i + 1];
            if is_selected {
                coloured_line = format!(
                    "{}{}{}",
                    coloured_line,
                    &part.on_dark_grey(),
                    &matching_char.on_dark_blue()
                );
            } else {
                coloured_line = format!(
                    "{}{}{}",
                    coloured_line,
                    &part,
                    &matching_char.on_dark_blue()
                );
            }
            start = i + 1;
        }
        let remaining_chars = &matching.long_name[start..matching.long_name.chars().count()];
        if is_selected {
            coloured_line = format!("{}{}", coloured_line, remaining_chars.on_dark_grey());
        } else {
            coloured_line = format!("{}{}", coloured_line, remaining_chars);
        }
        coloured_line
    }

    /// Renders the current result set
    pub fn render(&mut self) -> Result<()> {
        log::info!("render, console_offset: {}", self.console_offset);

        log::info!(
            "bottom: {}, top: {}",
            self.view.bottom_index,
            self.view.top_index
        );

        let matches = self.matches.as_ref().unwrap();

        self.view.update(matches);

        let view = self.view.contents.as_ref().unwrap();

        // Drop down so we don't over-write the terminal line that instigated
        // this run of lk.
        if self.first {
            for _ in 0..self.view.lines_to_show + 3 {
                writeln!(self.stdout, " ")?;
            }
            self.first = false
        }

        log::info!("Rendering lines ({}):", view.len());

        for (index, item) in view.iter().enumerate() {
            write!(
                self.stdout,
                "{}",
                termion::cursor::Goto(1, index as u16 + 1 + self.console_offset),
            )?;
            match item {
                Some(thing) => {
                    let fuzzy_indecies = &thing.score.as_ref().unwrap().1;

                    // Do some string manipulation to colourise the indexed parts
                    let coloured_line = UiState::get_coloured_line(
                        fuzzy_indecies,
                        thing,
                        index == self.view.selected_index as usize,
                    );
                    writeln!(
                        self.stdout,
                        "{}{}",
                        termion::clear::CurrentLine,
                        format!(
                            "{}-{}-{}-{} ",
                            self.view.selected_index,
                            self.matches.as_ref().unwrap().len(),
                            index,
                            coloured_line,
                        )
                    )?;
                }
                None => {
                    writeln!(self.stdout, "{}", termion::clear::CurrentLine,)?;
                }
            };
        }

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
            "{}{}{}>{} {}",
            termion::cursor::Show,
            termion::cursor::Goto(1, prompt_y + self.console_offset),
            color::Fg(color::Cyan),
            color::Fg(color::Reset),
            self.search_term
        )?;
        self.stdout.flush()?;

        Ok(())
    }
}

pub fn fuzzy_find_function(scripts: &[Script]) -> Result<Option<FuzzyFunction>> {
    let fuzzy_functions = scripts_to_flat(scripts);

    let mut state = UiState::new(fuzzy_functions);

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
        // escape by itself. That's what the 100 is below.
        // NB: some terminals might send these bytes too slowly and escape might not be caught.
        // NB: some terminals might use different escape keys entirely.
        if escaped == "^[" && instant.elapsed().as_micros() > 100 {
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
                    return if state.matches.is_some() {
                        Ok(Some(state.get_selected()))
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
    write!(state.stdout, "{}", termion::cursor::Show).unwrap();
    Ok(None)
}
#[derive(Clone)]
pub struct FuzzyFunction {
    long_name: String,
    pub script: Script,
    pub function: Function,
    score: Option<(i64, Vec<usize>)>,
}

fn scripts_to_flat(scripts: &[Script]) -> Vec<FuzzyFunction> {
    let mut fuzzy_functions: Vec<FuzzyFunction> = Vec::new();
    scripts.iter().for_each(|script| {
        script.functions.iter().for_each(|function| {
            fuzzy_functions.push(FuzzyFunction {
                function: function.to_owned(),
                script: script.clone(),
                long_name: format!(
                    "{}/{} - {}",
                    script.path(),
                    script.file_name(),
                    function.name
                ),
                // We'll set the search scores to 100 so we get the initial list displayed:
                // anything that has None has non match and doesn't get rendered.
                score: Some((100, Vec::new())),
            })
        })
    });
    fuzzy_functions
}
