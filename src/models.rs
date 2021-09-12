/// Everything we need to know about a function in a script
#[derive(PartialEq, Debug)]
pub struct Function {
    pub name: String,
    pub comment: Vec<String>,
}
