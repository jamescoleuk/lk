/// Everything we need to know about a function in a script
#[derive(PartialEq, Debug)]

pub struct Function {
    pub name: String,
    pub comment: Vec<String>,
}

pub struct Script {
    pub path: std::path::PathBuf,
    pub comment: Vec<String>,
    pub functions: Vec<Function>,
}

impl Script {
    pub const fn new(path: std::path::PathBuf) -> Self {
        Script {
            path,
            comment: Vec::new(),
            functions: Vec::new(),
        }
    }
}

pub struct ValidatedRequest {
    pub script_name: String,
    pub function_to_run: Option<String>,
}
