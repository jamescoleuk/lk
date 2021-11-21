// use crate::fuzzy::{Item, View};
// use anyhow::Result;
// use crossterm::style::Stylize;
// use fuzzy_matcher::skim::SkimMatcherV2;
// use fuzzy_matcher::FuzzyMatcher;
// use std::io::{stdout, Stdout, Write};
// use std::time::Instant;
// use termion::color;
// use termion::cursor::DetectCursorPos;
// use termion::event::Key;
// use termion::input::TermRead;
// use termion::raw::{IntoRawMode, RawTerminal};

// pub struct UiState<T>
// where
//     T: Clone,
// {
//     search_term: String,
//     fuzzy_functions: Vec<Item<T>>,
//     matches: Vec<Item<T>>,
//     console_offset: u16,
//     stdout: RawTerminal<Stdout>,
//     first: bool,
//     view: View<T>,
// }

// impl<T> UiState<T>
// where
//     T: Clone,
// {
//     pub fn new(functions: Vec<Item<T>>) -> Self {
//         // We need to know where to start rendering from. We can't do this later because
//         // we overwrite the cursor. Maybe we shouldn't do this? (TODO)
//         let mut stdout = stdout().into_raw_mode().unwrap();
//         let lines_to_show: i8 = 8;
//         let console_offset = if stdout.cursor_pos().is_ok() {
//             stdout.cursor_pos().unwrap().1 - (&lines_to_show + 2) as u16
//         } else {
//             log::error!("Cannot get cursor!");
//             0
//         };

//         let mut state = UiState {
//             search_term: String::from(""),
//             fuzzy_functions: functions,
//             matches: vec![],
//             console_offset,
//             stdout,
//             first: true,
//             view: View::new(lines_to_show),
//         };
//         state.update_matches();
//         state
//     }

//     pub fn up(&mut self) -> Result<()> {
//         log::info!("------------- up -------------");
//         let match_count = self.matches.len() as i8;
//         log::info!(
//             "selected_index: {}, match_count: {}, bottom_index: {}, top_index: {}",
//             self.view.selected_index,
//             match_count,
//             self.view.bottom_index,
//             self.view.top_index
//         );
//         // if self.selected_index > 0 && self.selected_index < match_count {
//         if self.view.selected_index > 0 {
//             log::info!("{} - {}", self.view.selected_index, match_count);
//             self.view.selected_index -= 1;
//         } else if self.view.top_index < (match_count - 1) as u8 {
//             log::info!("not going up because we're at the limit");
//             self.view.bottom_index += 1;
//             self.view.top_index += 1;
//         }
//         self.render()
//     }

//     pub fn down(&mut self) -> Result<()> {
//         log::info!("------------- down -------------");
//         let match_count = self.matches.len() as i8;
//         log::info!(
//             " selected_index: {}, match_count: {}, bottom_index: {}",
//             self.view.selected_index,
//             match_count,
//             self.view.bottom_index,
//         );
//         // if self.selected_index < match_count - 1 && self.selected_index >= self.bottom_index as i8 {

//         // Should we move the selection down?
//         if self.view.selected_index < self.view.top_index as i8 {
//             log::info!("incrementing");
//             self.view.selected_index += 1;
//         }

//         // Should we scroll down?
//         if self.view.selected_index > self.view.lines_to_show - 1 && self.view.bottom_index > 0 {
//             self.view.bottom_index -= 1;
//             self.view.top_index -= 1;
//             // if we've scrolled down then we don't want to change the selected index
//             // The selected index is for the view, so it stays the same.
//             if self.view.selected_index > 0 {
//                 self.view.selected_index -= 1;
//             }
//         } else {
//             log::info!("not scrolling down own because we're at the limit");
//         }

//         self.render()
//     }

//     pub fn append(&mut self, c: char) -> Result<()> {
//         log::info!("append");
//         // This is a normal key that we want to add to the search.
//         self.search_term = format!("{}{}", self.search_term, c);

//         let matcher = SkimMatcherV2::default();
//         for f in &mut self.fuzzy_functions {
//             f.score = matcher.fuzzy_indices(&f.name, &self.search_term);
//         }
//         self.update_matches();
//         self.render()
//     }

//     pub fn backspace(&mut self) -> Result<()> {
//         log::info!("backspace");
//         if self.search_term.chars().count() > 0 {
//             self.search_term =
//                 String::from(&self.search_term[..self.search_term.chars().count() - 1]);
//             let matcher = SkimMatcherV2::default();
//             for f in &mut self.fuzzy_functions {
//                 f.score = matcher.fuzzy_indices(&f.name, &self.search_term);
//             }
//         }
//         self.update_matches();
//         self.render()
//     }

//     pub fn get_selected(&self) -> &Item<T> {
//         let view = &mut self.view.contents.as_ref().unwrap();
//         let index = self.view.selected_index as usize;
//         &view[index]
//     }

//     /// Gets functions that match our current criteria, sorted by score.
//     fn update_matches(&mut self) {
//         log::info!("------------- update_matches -------------");
//         let mut matches = self
//             .fuzzy_functions
//             .iter()
//             .filter(|f| f.score.is_some())
//             .cloned()
//             .collect::<Vec<Item<T>>>();

//         // We want these in the order of their fuzzy matched score, i.e. closed matches
//         // matches.sort_by(|a, b| a.score.cmp(&b.score));
//         matches.sort_by(|a, b| b.score.cmp(&a.score));
//         let match_count = matches.len() as i8;
//         self.matches = matches;

//         // We can't have the index greater than the match count
//         if self.view.selected_index >= match_count {
//             self.view.selected_index = match_count - 1;
//             log::info!("resetting selected_index against match count");
//         }
//     }

//     /// Renders the current result set
//     pub fn render(&mut self) -> Result<()> {
//         log::info!("render, console_offset: {}", self.console_offset);

//         log::info!(
//             "bottom: {}, top: {}",
//             self.view.bottom_index,
//             self.view.top_index
//         );

//         self.view.update(&self.matches);

//         let view = self.view.contents.as_ref().unwrap();

//         // Drop down so we don't over-write the terminal line that instigated
//         // this run of lk.
//         if self.first {
//             for _ in 0..self.view.lines_to_show + 3 {
//                 writeln!(self.stdout, " ")?;
//             }
//             self.first = false
//         }

//         log::info!("Rendering lines ({}):", view.len());

//         for (index, item) in view.iter().enumerate() {
//             write!(
//                 self.stdout,
//                 "{}",
//                 termion::cursor::Goto(1, index as u16 + 1 + self.console_offset),
//             )?;
//             if item.is_blank {
//                 writeln!(self.stdout, "{}", termion::clear::CurrentLine,)?;
//             } else {
//                 let fuzzy_indecies = &item.score.as_ref().unwrap().1;

//                 // Do some string manipulation to colourise the indexed parts
//                 let coloured_line = get_coloured_line(
//                     &fuzzy_indecies,
//                     &item.name,
//                     index == self.view.selected_index as usize,
//                 );
//                 writeln!(
//                     self.stdout,
//                     "{}{}",
//                     termion::clear::CurrentLine,
//                     format!(
//                         "{}-{}-{}-{} ",
//                         self.view.selected_index,
//                         self.matches.len(),
//                         index,
//                         coloured_line,
//                     )
//                 )?;
//             }
//         }

//         // Render the prompt
//         let prompt_y = self.view.lines_to_show as u16 + 1;
//         let current_x = self.search_term.chars().count() + 2;

//         write!(
//             self.stdout,
//             "{}{}{}",
//             termion::clear::CurrentLine,
//             termion::cursor::Goto(current_x as u16, prompt_y + self.console_offset),
//             termion::clear::CurrentLine,
//         )?;
//         write!(
//             self.stdout,
//             "{}{}{}>{} {}",
//             termion::cursor::Show,
//             termion::cursor::Goto(1, prompt_y + self.console_offset),
//             color::Fg(color::Cyan),
//             color::Fg(color::Reset),
//             self.search_term
//         )?;
//         self.stdout.flush()?;

//         Ok(())
//     }

//     pub fn fuzzy_find_function(items: Vec<Item<T>>) -> Result<Option<T>> {
//         let mut state = UiState::new(items);

//         state.render()?;

//         let mut stdin = termion::async_stdin().keys();

//         // Run 'sed -n l' to explore escape codes
//         let mut escaped = String::from("");
//         let mut instant = Instant::now();

//         loop {
//             // What's going on here? The problem is how we detect escape.
//             // The key presses we're interested in, e.g. the arrows, are all preceded by escape, ^[.
//             // E.g. up is ^[[A and down is ^[[B. So the question is how do we identify an escape
//             // key by itself? If it's ^[[A then that's ^[ followed almost instantly by [A. If we have
//             // ^[ followed by a pause then we know it's not an escape for some other key, but an
//             // escape by itself. That's what the 100 136His below.
//             // NB: some terminals might send these bytes too slowly and escape might not be caught.
//             // NB: some terminals might use different escape keys entirely.
//             if escaped == "^[" && instant.elapsed().as_micros() > 100 {
//                 break;
//             }

//             if let Some(Ok(key)) = stdin.next() {
//                 match key {
//                     // ctrl-c and ctrl-d are two ways to exit.
//                     Key::Ctrl('c') => break,
//                     Key::Ctrl('d') => break,

//                     // NB: It'd be neat if we could use Key::Up and Key::Down but they don't
//                     // work in raw mode. So we've got to deal with the escape codes manually.

//                     // This captures the enter key
//                     Key::Char('\n') => {
//                         return if state.matches.len() > 0 {
//                             Ok(Some(state.get_selected().item.as_ref().unwrap().to_owned()))
//                         } else {
//                             Ok(None)
//                         };
//                     }
//                     Key::Char(c) => {
//                         if !escaped.is_empty() {
//                             escaped = format!("{}{}", escaped, c);
//                             match escaped.as_str() {
//                                 "^[" => continue,
//                                 "^[[" => continue,
//                                 "^[[A" => {
//                                     escaped = String::from("");
//                                     state.up()?;
//                                 }
//                                 "^[[B" => {
//                                     escaped = String::from("");
//                                     state.down()?;
//                                 }
//                                 _ => {
//                                     // This is nothing we recognise so let's abandon the escape sequence.
//                                     escaped = String::from("");
//                                 }
//                             }
//                         } else {
//                             state.append(c)?;
//                         }
//                     }
//                     Key::Esc => {
//                         // All we're doing here is recording that we've entered an escape sequence.
//                         // It's actually handled when we handle chars.
//                         if escaped.is_empty() {
//                             escaped = String::from("^[");
//                             instant = Instant::now();
//                         }
//                     }
//                     Key::Backspace => {
//                         state.backspace()?;
//                     }
//                     _ => {}
//                 }
//                 state.stdout.flush().unwrap();
//             }
//         }
//         write!(state.stdout, "{}", termion::cursor::Show).unwrap();
//         Ok(None)
//     }
// }

// fn get_coloured_line(fuzzy_indecies: &[usize], text: &str, is_selected: bool) -> String {
//     // Do some string manipulation to colourise the indexed parts
//     let mut coloured_line = String::from("");
//     let mut start = 0;
//     for i in fuzzy_indecies {
//         let part = &text[start..*i];
//         let matching_char = &text[*i..*i + 1];
//         if is_selected {
//             coloured_line = format!(
//                 "{}{}{}",
//                 coloured_line,
//                 &part.on_dark_grey(),
//                 &matching_char.on_dark_blue()
//             );
//         } else {
//             coloured_line = format!(
//                 "{}{}{}",
//                 coloured_line,
//                 &part,
//                 &matching_char.on_dark_blue()
//             );
//         }
//         start = i + 1;
//     }
//     let remaining_chars = &text[start..text.chars().count()];
//     if is_selected {
//         coloured_line = format!("{}{}", coloured_line, remaining_chars.on_dark_grey());
//     } else {
//         coloured_line = format!("{}{}", coloured_line, remaining_chars);
//     }
//     coloured_line
// }
