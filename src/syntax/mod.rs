pub mod highlight;

use tree_sitter::{Parser, Tree};

pub struct SyntaxEngine {
    parser: Parser,
    tree: Option<Tree>,
}

impl SyntaxEngine {
    pub fn new() -> Self {
        let mut parser = Parser::new();

        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Failed to load Rust grammar");

        Self { parser, tree: None }
    }

    pub fn parse(&mut self, source: &str) {
        self.tree = self.parser.parse(source, None);
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }
}
