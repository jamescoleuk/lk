use super::item::Item;

pub struct List<T>
where
    T: Clone,
{
    pub top_index: u8,
    pub bottom_index: u8,
    pub lines_to_show: i8,
    pub selected_index: i8,
    pub contents: Vec<Item<T>>,
}

impl<T> List<T>
where
    T: Clone,
{
    pub fn new(lines_to_show: i8) -> Self {
        List {
            contents: vec![],
            top_index: lines_to_show as u8 - 1,
            selected_index: (lines_to_show - 1) as i8,
            lines_to_show,
            bottom_index: 0,
        }
    }

    pub fn up(&mut self, matches: &[Item<T>]) {
        let match_count = matches.len() as i8;
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.top_index < (match_count - 1) as u8 {
            self.bottom_index += 1;
            self.top_index += 1;
        }
        self.floor_selected_index();
    }

    pub fn down(&mut self) {
        // Should we move the selection down?
        if self.selected_index < self.top_index as i8 {
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
        }
        self.floor_selected_index();
    }

    fn floor_selected_index(&mut self) {
        let index_of_first_blank = self.contents.iter().rev().position(|item| item.is_blank);
        if let Some(rev_index) = index_of_first_blank {
            let index = self.lines_to_show - rev_index as i8;
            if self.selected_index < index as i8 {
                self.selected_index = index
            }
        }
    }

    /// Takes the current matches and updates the visible contents.
    pub fn update(&mut self, matches: &[Item<T>]) {
        log::info!("Updating view with {} match(es)", matches.len());
        let mut to_render: Vec<Item<T>> = Vec::new();
        // Get everything in our display window
        for i in self.bottom_index..self.top_index + 1 {
            if matches.len() > (i).into() {
                to_render.push(matches[i as usize].clone());
            } else {
                to_render.push(Item::empty());
            }
        }
        to_render.reverse();

        self.contents = to_render;
        self.floor_selected_index();
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
        view: List<TestItem>,
    }

    impl Setup {
        fn new(lines_to_show: i8) -> Self {
            let view = List::<TestItem>::new(lines_to_show);

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
                view,
            }
        }
    }

    #[test]
    fn test_update() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.update(&setup.items);

        // THEN
        assert_eq!(setup.view.contents.len(), 8);
        assert_eq!(setup.view.selected_index, 7); // 0-indexed
        assert_eq!(setup.view.get_selected().item.as_ref().unwrap().name, "A")
    }

    #[test]
    fn test_up() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.up(&setup.items); // 6
        setup.view.up(&setup.items); // 5
        setup.view.up(&setup.items); // 4

        // THEN
        assert_eq!(setup.view.contents.len(), 8);
        assert_eq!(setup.view.selected_index, 4);
    }

    #[test]
    fn test_up_to_extremis() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        // More than lines_to_show
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);
        setup.view.up(&setup.items);

        // THEN
        assert_eq!(setup.view.contents.len(), 8);
        assert_eq!(setup.view.selected_index, 0);
    }

    #[test]
    fn test_down_at_bottom() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.down(); // 7

        // THEN
        assert_eq!(setup.view.contents.len(), 8);
        assert_eq!(setup.view.selected_index, 7);
    }

    #[test]
    fn test_down() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.up(&setup.items); // 6
        setup.view.up(&setup.items); // 5
        setup.view.up(&setup.items); // 4
        setup.view.down(); // 5

        // THEN
        assert_eq!(setup.view.contents.len(), 8);
        assert_eq!(setup.view.selected_index, 5);
    }

    #[test]
    fn test_few() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.update(&setup.few_items);
        setup.view.up(&setup.few_items); // 6
        setup.view.up(&setup.few_items); // 5
        setup.view.up(&setup.few_items); // 5
        setup.view.up(&setup.few_items); // 5

        // THEN
        assert_eq!(setup.view.contents.len(), 8); // Still 8, but blanks
        assert_eq!(setup.view.selected_index, 5);
    }
}
