#[derive(Clone)]
pub struct Item<T>
where
    T: Clone,
{
    pub is_blank: bool,
    pub name: String,
    pub score: (i64, Vec<usize>),
    pub item: Option<T>,
}

pub struct View<T>
where
    T: Clone,
{
    pub top_index: u8,
    pub bottom_index: u8,
    pub lines_to_show: i8,
    pub selected_index: i8,
    //TODO: don't use a FuzzyFunction, but rather a trait which FuzzyFunction can implement
    // pub contents: Option<Vec<Option<FuzzyFunction>>>,
    pub contents: Option<Vec<Item<T>>>,
}

impl<T> View<T>
where
    T: Clone,
{
    pub fn new(lines_to_show: i8) -> Self {
        View {
            contents: Option::None,
            top_index: lines_to_show as u8 - 1,
            selected_index: (lines_to_show - 1) as i8,
            lines_to_show,
            bottom_index: 0,
        }
    }

    pub fn up(&mut self, matches: &Vec<Item<T>>) {
        log::info!("------------- up -------------");
        let match_count = matches.len() as i8;
        log::info!(
            "selected_index: {}, match_count: {}, bottom_index: {}, top_index: {}",
            self.selected_index,
            match_count,
            self.bottom_index,
            self.top_index
        );
        // if self.selected_index > 0 && self.selected_index < match_count {
        if self.selected_index > 0 {
            log::info!("{} - {}", self.selected_index, match_count);
            self.selected_index -= 1;
        } else if self.top_index < (match_count - 1) as u8 {
            log::info!("not going up because we're at the limit");
            self.bottom_index += 1;
            self.top_index += 1;
        }
        // self.render()
    }

    /// Takes the current matches and updates the visible contents.
    // pub fn update(&mut self, matches: &[FuzzyFunction]) {
    pub fn update(&mut self, matches: &[Item<T>]) {
        let mut to_render: Vec<Item<T>> = Vec::new();
        // Get everything in our display window
        for i in self.bottom_index..self.top_index + 1 {
            if matches.len() > (i).into() {
                // to_render.push(Option::Some(matches[i as usize].clone()));
                to_render.push(matches[i as usize].clone());
            } else {
                to_render.push(Item {
                    is_blank: true,
                    name: "".to_string(),
                    score: (0, vec![]),
                    item: None,
                });
                // to_render.push(Option::None);
            }
        }

        // Now that the order is reversed our indexes will match. If the selected_index
        // is outside the range of what's selectable, i.e. our matches, then we need
        // to gently reset it back to the limit. This prevents the selection going onto
        // blank lines and also moves the selection to the top of the matches when
        // the number of matches shrinks.
        for (i, item) in to_render.iter().enumerate() {
            if item.is_blank {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::script::{Function, Script};

    use super::*;

    #[test]
    fn test_up() {
        // Given
        let mut view = View::<FuzzyFunction>::new(8);
        //TODO This is way too long and complicated. It's the domain model's struct
        //     not the view's struct. I need a trait (Item) and have FuzzyFunction
        //     implement that. That'll maek this much easier to stub, and then I can
        //     properly test the up and and so on, without it all being tied into the
        //     bloody display! When the tests pass it works. That'll be fab.
        let items = vec![Item {
            is_blank: false,
            score: (0, vec![]),
            name: "some name".to_string(),
            item: None,
        }];
        // let matches = vec![FuzzyFunction {
        //     long_name: String::from("foo1"),
        //     script: Script {
        //         path: PathBuf::new(),
        //         absolute_path: PathBuf::new(),
        //         comment: vec!["Some".to_string(), "Comment".to_string()],
        //         functions: vec![Function {
        //             name: String::from("some function name"),
        //             comment: vec!["comment".to_string()],
        //         }],
        //     },
        //     function: Function {
        //         name: String::from("some function name"),
        //         comment: vec!["comment".to_string()],
        //     },
        //     score: Some((100, Vec::new())),
        // }];
        view.update(&items);

        // WHen
        view.up(&items);

        // Then
        let contents = view.contents.unwrap();
        assert_eq!(contents.len(), 1);
        // assert_eq!(clean_comment_line("#First line"), "First line");
    }
}
