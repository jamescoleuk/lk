use crate::script::{Function, Script};
use anyhow::Result;
use crossterm::style::Stylize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdin, stdout, Write};
use std::time::{Duration, Instant};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

struct UiState {
    search_term: String,
    selected_index: i8,
    lines_to_show: i8,
    fuzzy_functions: Vec<FuzzyFunction>,
    matches: Option<Vec<FuzzyFunction>>,
    top_index: u8,
    bottom_index: u8,
}

impl UiState {
    pub fn new(functions: Vec<FuzzyFunction>) -> Self {
        let lines_to_show = 8;
        let mut state = UiState {
            search_term: String::from(""),
            selected_index: lines_to_show - 1,
            lines_to_show,
            fuzzy_functions: functions,
            matches: Option::None,
            top_index: 0,
            bottom_index: lines_to_show as u8 - 1,
        };
        state.update_matches();
        state
    }

    pub fn up(&mut self) -> Result<()> {
        log::info!("up");
        let match_count = self.matches.as_ref().unwrap().len() as i8;
        if self.selected_index > 0 && self.selected_index < match_count {
            println!("{} - {}", self.selected_index, match_count);
            self.selected_index -= 1;
        }
        self.render()
    }

    pub fn down(&mut self) -> Result<()> {
        log::info!("down");
        let match_count = self.matches.as_ref().unwrap().len() as i8;
        if self.selected_index >= 0 && self.selected_index < match_count - 1 {
            println!("{} - {}", self.selected_index, match_count);
            self.selected_index += 1;
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

    /// Gets functions that match our current criteria, sorted by score.
    fn update_matches(&mut self) {
        log::info!("update_matches");
        let mut matches = self
            .fuzzy_functions
            .iter()
            .filter(|f| f.score.is_some())
            .cloned()
            .collect::<Vec<FuzzyFunction>>();

        // We want these in the order of their fuzzy matched score, i.e. closed matches
        matches.sort_by(|a, b| a.score.cmp(&b.score));
        let match_count = matches.len() as i8;
        self.matches = Some(matches);

        // We can't have the index greater than the match count
        if self.selected_index >= match_count {
            println!("shrinking");
            self.selected_index = match_count - 1;
        }
    }

    /// Gets the number of blank lines we need to display, given the current match set
    fn blank_lines(&self) -> i8 {
        log::info!("blank_lines");
        let match_count = self.matches.as_ref().unwrap().len() as i8;
        // Figure out how many blank lines we need to show at the top
        if self.lines_to_show >= match_count {
            self.lines_to_show - match_count
        } else {
            0
        }
    }

    fn get_coloured_line(
        fuzzy_indecies: &Vec<usize>,
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
    pub fn render(&self) -> Result<()> {
        log::info!("render");
        let mut stdout = stdout().into_raw_mode()?;

        let mut to_render: Vec<Option<FuzzyFunction>> = Vec::new();
        let matches = self.matches.as_ref().unwrap();

        println!("bottom: {}, top: {}", self.bottom_index, self.top_index);
        // Add blank lines
        for _ in 0..self.blank_lines() {
            to_render.push(Option::None)
        }
        // If we've got fewer than the lines to show we'll just add everything
        if matches.len() < self.lines_to_show as usize {
            log::info!("showing everything we've got");
            for m in matches.iter() {
                to_render.push(Option::Some(m.clone()));
            }
        } else {
            log::info!("showing subset");
            // Otherwise we need to add a slice of the matches based on top and bottom indecies.
            for i in self.top_index..self.bottom_index + 1 {
                let func = matches[i as usize].clone();
                to_render.push(Option::Some(func));
            }
        }

        log::info!("about to render {} items", to_render.len());
        // Render the searched lines
        for (index, item) in to_render.iter().enumerate() {
            match item {
                Some(thing) => {
                    let fuzzy_indecies = &thing.score.as_ref().unwrap().1;

                    // Do some string manipulation to colourise the indexed parts
                    let coloured_line = UiState::get_coloured_line(
                        fuzzy_indecies,
                        thing,
                        index == self.selected_index as usize,
                    );
                    writeln!(
                        stdout,
                        "{}{}{}{}",
                        termion::cursor::Hide,
                        termion::cursor::Goto(1, index as u16 + 1),
                        // termion::cursor::Goto(1, index as u16),
                        termion::clear::CurrentLine,
                        format!(
                            "{}-{}-{}-{} ",
                            // count,
                            self.selected_index,
                            self.matches.as_ref().unwrap().len(),
                            index,
                            coloured_line,
                        )
                    )?;
                }
                None => {
                    writeln!(
                        stdout,
                        "{}{}{}",
                        termion::cursor::Hide,
                        termion::cursor::Goto(1, index as u16),
                        termion::clear::CurrentLine,
                    )?;
                }
            };
        }

        // Render the prompt
        let prompt_y = self.lines_to_show as u16 + 1;
        let current_x = self.search_term.chars().count() + 2;
        write!(
            stdout,
            "{}{}{}",
            termion::clear::CurrentLine,
            termion::cursor::Goto(current_x as u16, prompt_y),
            termion::clear::CurrentLine,
        )?;
        write!(
            stdout,
            "{}{}{}>{} {}",
            termion::cursor::Show,
            termion::cursor::Goto(1, prompt_y),
            color::Fg(color::Cyan),
            color::Fg(color::Reset),
            self.search_term
        )?;
        stdout.flush()?;
        Ok(())
    }
}

pub fn fuzzy_find_function(scripts: &[Script]) -> Result<()> {
    let fuzzy_functions = scripts_to_flat(scripts);

    let mut stdout = stdout().into_raw_mode()?;

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
                    // TODO: if there's something selected
                    //           run the thing
                    //       else
                    //           nothing, or a watnin
                    break;
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
            stdout.flush().unwrap();
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    Ok(())
}
#[derive(Clone)]
struct FuzzyFunction {
    long_name: String,
    script: Script,
    function: Function,
    score: Option<(i64, Vec<usize>)>,
    //TODO: add is_selected!
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
                // score: Option::None,
                // We'll set the search scores to 100 so we get the initial list displayed:
                // anything that has None has non match and doesn't get rendered.
                score: Some((100, Vec::new())),
            })
        })
    });
    fuzzy_functions
}
