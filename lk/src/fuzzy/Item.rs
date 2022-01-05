/// An Item in the list the user sees when searching.

#[derive(Clone)]
pub struct Item<T>
where
    T: Clone,
{
    /// This is a filler item: there isn't a search result in this place.
    pub is_blank: bool,
    pub name: String,
    pub score: Option<(i64, Vec<usize>)>,
    pub item: Option<T>,
}

impl<T> Item<T>
where
    T: Clone,
{
    /// Any 'new' item is always non-blank, because it has a name.
    /// Use 'empty' to create a blank item.
    pub fn new(name: String, item: T) -> Self {
        Item::<T> {
            is_blank: false, // Any 'new' Item is always a non-blank.
            name,
            item: Some(item),
            score: None, // It won't be scored yet.
        }
    }

    /// Creates a blank item to fill in the visual space in the list.
    /// Never has an actual item attached, or a score, or a name.
    pub fn empty() -> Self {
        Item::<T> {
            is_blank: true,
            name: "".to_string(),
            score: None,
            item: None,
        }
    }
}
