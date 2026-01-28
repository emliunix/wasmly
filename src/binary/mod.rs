pub mod leb128;
pub mod primitives;
pub mod sections;
pub mod instructions;
pub mod error;

pub use error::BinaryError;
pub use primitives::ParseResult;

pub struct Module {
    pub types: Vec<types::Ty>,
    pub functions: Vec<types::Func>,
    pub imports: Vec<types::Import>,
    pub exports: Vec<types::Export>,
    pub start: Option<usize>,
}

pub type Func = (Vec<types::Ty>, Vec<types::Ty>);

impl Module {
    pub fn new() -> Self {
        Self {
            types: vec![],
            functions: vec![],
            imports: vec![],
            exports: vec![],
            start: None,
        }
    }
}
