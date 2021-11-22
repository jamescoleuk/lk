use super::item::Item;

pub struct View<T>
where
    T: Clone,
{
    pub top_index: u8,
    pub bottom_index: u8,
    pub lines_to_show: i8,
    pub selected_index: i8,
    pub contents: Vec<Item<T>>,
}

impl<T> View<T>
where
    T: Clone,
{
    pub fn new(lines_to_show: i8) -> Self {
        View {
            contents: vec![],
            top_index: lines_to_show as u8 - 1,
            selected_index: (lines_to_show - 1) as i8,
            lines_to_show,
            bottom_index: 0,
        }
    }

    pub fn up(&mut self, matches: &[Item<T>]) {
        log::info!("------------- UP -------------");
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
    }

    pub fn down(&mut self, matches: &[Item<T>]) {
        log::info!("------------- down -------------");
        let match_count = matches.len() as i8;
        log::info!(
            " selected_index: {}, match_count: {}, bottom_index: {}",
            self.selected_index,
            match_count,
            self.bottom_index,
        );
        // if self.selected_index < match_count - 1 && self.selected_index >= self.bottom_index as i8 {

        // Should we move the selection down?
        if self.selected_index < self.top_index as i8 {
            log::info!("incrementing");
            self.selected_index += 1;
        }

        // Should we scroll down?
        if self.selected_index > self.lines_to_show - 1 && self.bottom_index > 0 {
            self.bottom_index -= 1;
            self.top_index -= 1;
            // if we've scrolled down then we don't want to change the selected index
            // The selected index is for the view, so it stays the same.
            if self.selected_index > 0 {
                self.selected_index -= 1;
            }
        } else {
            log::info!("not scrolling down own because we're at the limit");
        }
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
                to_render.push(Item::empty());
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
        self.contents = to_render;
    }

    pub fn get_selected(&self) -> &Item<T> {
        let index = self.selected_index as usize;
        &self.contents[index]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone)]
    struct TestItem {
        name: String,
    }

    fn item(name: &str) -> Item<TestItem> {
        Item::new(
            String::from(name),
            TestItem {
                name: String::from(name),
            },
        )
    }

    struct Setup {
        items: Vec<Item<TestItem>>,
        few_items: Vec<Item<TestItem>>,
    }

    impl Setup {
        fn new() -> Self {
            Setup {
                items: vec![
                    item("A"),
                    item("B"),
                    item("C"),
                    item("D"),
                    item("E"),
                    item("F"),
                    item("G"),
                    item("H"),
                    item("I"),
                    item("J"),
                    item("K"),
                    item("L"),
                    item("M"),
                ],
                few_items: vec![item("A"), item("B"), item("C")],
            }
        }
    }

    #[test]
    fn test_update() {
        // GIVEN
        let lines_to_show = 8;
        let mut view = View::<TestItem>::new(lines_to_show);
        let setup = Setup::new();

        // WHEN
        view.update(&setup.items);

        // THEN
        assert_eq!(view.contents.len(), lines_to_show as usize);
        assert_eq!(view.selected_index, lines_to_show - 1);
        assert_eq!(view.get_selected().item.as_ref().unwrap().name, "A")
    }

    #[test]
    fn test_up() {
        // GIVEN
        let lines_to_show = 8;
        let mut view = View::<TestItem>::new(lines_to_show);
        let setup = Setup::new();
        view.update(&setup.items);

        // WHEN
        view.up(&setup.items); // 6
        view.up(&setup.items); // 5
        view.up(&setup.items); // 4

        // THEN
        let contents = view.contents;
        assert_eq!(contents.len(), lines_to_show as usize);
        assert_eq!(view.selected_index, 4);
    }

    #[test]
    fn test_up_to_extremis() {
        // GIVEN
        let lines_to_show = 8;
        let mut view = View::<TestItem>::new(lines_to_show);
        let setup = Setup::new();
        view.update(&setup.items);

        // WHEN
        // More than lines_to_show
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);
        view.up(&setup.items);

        // THEN
        let contents = view.contents;
        assert_eq!(contents.len(), lines_to_show as usize);
        assert_eq!(view.selected_index, 0);
    }

    #[test]
    fn test_down_at_bottom() {
        // GIVEN
        let lines_to_show = 8;
        let mut view = View::<TestItem>::new(lines_to_show);
        let setup = Setup::new();
        view.update(&setup.items);

        // WHEN
        view.down(&setup.items); // 7

        // THEN
        let contents = view.contents;
        assert_eq!(contents.len(), lines_to_show as usize);
        assert_eq!(view.selected_index, 7);
    }

    #[test]
    fn test_down() {
        // GIVEN
        let lines_to_show = 8;
        let mut view = View::<TestItem>::new(lines_to_show);
        let setup = Setup::new();
        view.update(&setup.items);

        // WHEN
        view.up(&setup.items); // 6
        view.up(&setup.items); // 5
        view.up(&setup.items); // 4
        view.down(&setup.items); // 5

        // THEN
        let contents = view.contents;
        assert_eq!(contents.len(), lines_to_show as usize);
        assert_eq!(view.selected_index, 5);
    }
}
