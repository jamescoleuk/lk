use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use ratatui::style::Stylize as _;
use ratatui::widgets::ListState;
use ratatui::{style::Color, text::Span};

use crate::script::{self, Function, Script};

/// This struct holds the state of a list widget.
pub(crate) struct StatefulList {
    pub(crate) state: ListState,
    pub(crate) items: Vec<Item>,
}

impl StatefulList {
    fn with_items(items: Vec<Item>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) if i >= self.items.len() - 1 => 0,
            Some(i) => i + 1,
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(0) | None => self.items.len() - 1,
            Some(i) => i - 1,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn get_as_coloured(&self) -> Vec<Vec<Span<'_>>> {
        self.items.iter().map(Item::coloured).collect()
    }
}

/// An item represents an individual function in a script which we want to display in our list.
/// So it contains everything we need to know about this function.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) source: (Script, Function),
    pub(crate) score: Option<(i64, Vec<usize>)>,
}

impl Item {
    /// Highlights the line for this function. Will highlight matching characters in the search.
    pub fn coloured(&self) -> Vec<Span<'_>> {
        let mut spans: Vec<Span> = vec![];
        let text = self.name.clone();
        let text_vec = text.chars().collect::<Vec<_>>();
        let mut start = 0;
        if let Some(score) = &self.score {
            for i in &score.1 {
                let part = text_vec[start..*i].iter().collect::<String>().clone();
                let matching_char = text_vec[*i..*i + 1].iter().collect::<String>().clone();
                spans.push(Span::from(part));
                spans.push(Span::from(matching_char).fg(Color::Red));
                start = i + 1;
            }
        };
        let remaining_chars = text_vec[start..text.chars().count()]
            .iter()
            .collect::<String>();

        spans.push(Span::from(remaining_chars));
        spans
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub(crate) struct App {
    items: StatefulList,
    pub(crate) filtered_items: StatefulList,
    pub(crate) search_term: String,
}

impl App {
    pub fn from(scripts: &[script::Script]) -> App {
        let items: Vec<Item> = scripts
            .iter()
            .flat_map(|script| {
                script.functions.iter().map(move |function| Item {
                    name: format!("{}/{}", script.path.display(), function.name),
                    score: None,
                    source: (script.clone(), function.clone()),
                })
            })
            .collect();
        App {
            items: StatefulList::with_items(items.clone()),
            filtered_items: StatefulList::with_items(items),
            search_term: String::new(),
        }
    }

    pub fn update_search_term(&mut self, term: &str) {
        self.search_term.push_str(term);
        self.update_items()
    }

    pub fn delete_search_term_char(&mut self) {
        self.search_term.pop();
        self.update_items()
    }

    fn update_items(&mut self) {
        log::info!("search term: {}", self.search_term);

        // First generate the scores
        let matcher = SkimMatcherV2::default();
        self.items
            .items
            .iter_mut()
            .for_each(|item| item.score = matcher.fuzzy_indices(&item.name, &self.search_term));

        // Then filter the items, but only if we have a search term
        if self.search_term.is_empty() {
            self.filtered_items = StatefulList::with_items(self.items.items.clone());
        } else {
            self.filtered_items = StatefulList::with_items(
                self.items
                    .items
                    .iter()
                    .filter(|item| match item.score.clone() {
                        Some((_, indices)) => !indices.is_empty(),
                        None => false,
                    })
                    .cloned()
                    .collect(),
            );
        }

        // We also need to update our selection, othewise if we go from a short list
        // to a long list we might lose out selection somewhere below in a non-visible area.
    }

    pub fn get_selected(&self) -> Option<&Item> {
        self.filtered_items
            .state
            .selected()
            .and_then(|i| self.filtered_items.items.get(i))
    }
}
