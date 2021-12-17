#[derive(Clone)]
pub struct Item<T>
where
    T: Clone,
{
    pub is_blank: bool,
    pub name: String,
    pub score: Option<(i64, Vec<usize>)>,
    pub item: Option<T>,
}

impl<T> Item<T>
where
    T: Clone,
{
    pub fn new(name: String, item: T) -> Self {
        Item::<T> {
            is_blank: false,
            name,
            item: Some(item),
            score: None,
        }
    }

    pub fn empty() -> Self {
        Item::<T> {
            is_blank: true,
            name: "".to_string(),
            score: None,
            item: None,
        }
    }
}
