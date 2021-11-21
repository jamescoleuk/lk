use crossterm::style::Stylize;

pub mod Item;
pub mod UiState;
pub mod View;
// use crate::fuzzy::Item::Item;

fn get_coloured_line(fuzzy_indecies: &[usize], text: &str, is_selected: bool) -> String {
    // Do some string manipulation to colourise the indexed parts
    let mut coloured_line = String::from("");
    let mut start = 0;
    for i in fuzzy_indecies {
        let part = &text[start..*i];
        let matching_char = &text[*i..*i + 1];
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
    let remaining_chars = &text[start..text.chars().count()];
    if is_selected {
        coloured_line = format!("{}{}", coloured_line, remaining_chars.on_dark_grey());
    } else {
        coloured_line = format!("{}{}", coloured_line, remaining_chars);
    }
    coloured_line
}
