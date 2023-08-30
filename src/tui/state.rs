use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::widgets::*;

use crate::script::{self, Function, Script};

pub(crate) struct StatefulList<T> {
    pub(crate) state: ListState,
    pub(crate) items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) source: (Script, Function),
    pub(crate) score: Option<(i64, Vec<usize>)>,
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is
/// a wrapper around `ListState`. Keeping track of the items state let us render the associated
/// widget with its state and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub(crate) struct App {
    items: StatefulList<Item>,
    pub(crate) filtered_items: StatefulList<Item>,
    pub(crate) search_term: String,
}

impl App {
    pub fn from(scripts: &[script::Script]) -> App {
        let mut items: Vec<Item> = Vec::new();
        scripts.iter().for_each(|script| {
            script.functions.iter().for_each(|function| {
                items.push(Item {
                    name: function.name.clone(),
                    score: None,
                    source: (script.clone(), function.clone()),
                })
            })
        });
        App {
            items: StatefulList::with_items(items.clone()),
            filtered_items: StatefulList::with_items(items.clone()),
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
        match self.filtered_items.state.selected() {
            Some(i) => self.filtered_items.items.get(i),
            None => None,
        }
    }
}
