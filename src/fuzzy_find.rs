use anyhow::Result;
use colored::Colorize;
use termion::color;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{cursor::DetectCursorPos, event::Key};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::script::{Function, Script};
use std::io::{stdin, stdout, Write};

pub fn fuzzy_find_function(scripts: &Vec<Script>) -> Result<()> {
    let mut fuzzy_functions = scripts_to_flat(scripts);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;

    render(&"", &fuzzy_functions)?;

    let mut search_term = String::from("");
    for c in stdin.keys() {
        render(&search_term, &fuzzy_functions)?;

        match c.unwrap() {
            Key::Ctrl('c') => break,
            Key::Ctrl('d') => break,
            Key::Esc => break,
            Key::Char('\n') => {
                // TODO: if there's something selected
                //           run the thing
                //       else
                //           nothing, or a watnin
                break;
            }
            Key::Char(c) => {
                search_term = format!("{}{}", search_term, c);

                let matcher = SkimMatcherV2::default();
                for f in &mut fuzzy_functions {
                    f.score = matcher.fuzzy_indices(&f.long_name, &search_term);
                }
                render(&search_term, &fuzzy_functions)?;

                for (i, f) in fuzzy_functions.iter().enumerate() {
                    if f.score.is_some() {
                        render(&search_term, &fuzzy_functions)?;
                    }
                }
                // fuzzy_functions.iter().for_each(|f| {
                //     //&&
                //     f.score = matcher.fuzzy_match(&f.long_name, &search_term);
                // })
            }
            // TODO: Use these to move the cursor about. Challenge: termion can only
            //       clear from the cursor. It can't clear onr char! I could save what's
            //       there and write it back, but this feels like a more advanced feature.
            // Key::Left => {
            //     let new_pos = stdout.cursor_pos()?.0 - 1;
            //     write!(stdout, "{}", termion::cursor::Goto(new_pos, starting_y),)?;
            // }
            // Key::Right => println!("→"),
            // Use these to select the thing
            // Key::Up => println!("↑"),
            // Key::Down => println!("↓"),
            Key::Backspace => {
                if search_term.chars().count() > 0 {
                    search_term = String::from(&search_term[..search_term.chars().count() - 1]);
                    let matcher = SkimMatcherV2::default();
                    for f in &mut fuzzy_functions {
                        f.score = matcher.fuzzy_indices(&f.long_name, &search_term);
                    }
                    render(&search_term, &fuzzy_functions)?;
                }
            }
            _ => {}
        }
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
    Ok(())
}

fn render(search_term: &str, fuzzy_functions: &Vec<FuzzyFunction>) -> Result<()> {
    let mut stdout = stdout().into_raw_mode()?;
    // We only care about functions that have been scored, i.e. ones that match.
    let mut matches: Vec<&FuzzyFunction> = fuzzy_functions
        .iter()
        .filter(|f| f.score.is_some())
        .collect();

    // We want these in the order of their fuzzy matched score, i.e. closed matches
    matches.sort_by(|a, b| a.score.cmp(&b.score));

    // This is how many lines of results we want to show. We might want to make this customisable.
    let lines_to_show = 10;
    let mut current_line = 0;

    // Figure out how many blank lines we need to show at the top
    let blank_lines = if lines_to_show >= matches.len() {
        lines_to_show - matches.len()
    } else {
        0
    };

    // Render those blank lines
    for _ in 0..blank_lines {
        write!(
            stdout,
            "{}{}{}{}\n",
            termion::cursor::Hide,
            termion::cursor::Goto(1, current_line as u16),
            termion::clear::CurrentLine,
            ""
        )?;
        current_line += 1;
    }
    // Render the remaining lines
    for matching in matches.iter() {
        // Make sure we only show the top
        if lines_to_show > current_line {
            let fuzzy_indecies = &matching.score.as_ref().unwrap().1;

            // Do some string manipulation to colourise the indexed parts
            let mut coloured_line = String::from("");
            let mut start = 0;
            for i in fuzzy_indecies {
                let part = &matching.long_name[start..*i];
                let matching_char = &matching.long_name[*i..*i + 1];
                coloured_line = format!("{}{}{}", coloured_line, &part, &matching_char.on_blue());
                start = i + 1;
            }
            let remaining_chars = &matching.long_name[start..matching.long_name.chars().count()];
            coloured_line = format!("{}{}", coloured_line, remaining_chars);

            write!(
                stdout,
                "{}{}{}{}\n",
                termion::cursor::Hide,
                termion::cursor::Goto(1, current_line as u16),
                termion::clear::CurrentLine,
                format!("{} ", coloured_line,)
            )?;
            current_line += 1;
        }
    }

    let current = stdout.cursor_pos()?.1;
    let current_x = search_term.chars().count() + 2;
    write!(
        stdout,
        "{}{}",
        termion::clear::CurrentLine,
        termion::cursor::Goto(current_x as u16, current)
    )?;
    write!(
        stdout,
        "{}{}{}>{} {}",
        termion::cursor::Show,
        termion::cursor::Goto(1, current),
        color::Fg(color::Cyan),
        color::Fg(color::Reset),
        search_term
    )?;
    stdout.flush()?;
    Ok(())
}

struct FuzzyFunction {
    long_name: String,
    script: Script,
    function: Function,
    score: Option<(i64, Vec<usize>)>,
}

fn scripts_to_flat(scripts: &Vec<Script>) -> Vec<FuzzyFunction> {
    let mut fuzzy_functions: Vec<FuzzyFunction> = Vec::new();
    scripts.iter().for_each(|script| {
        script.functions.iter().for_each(|function| {
            fuzzy_functions.push(FuzzyFunction {
                function: function.to_owned(),
                script: script.clone().to_owned(),
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

fn get_matching_functions() {}
