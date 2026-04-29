use syn::{
    File, ItemConst,
    visit::{self, Visit},
};

/// Find an enum in a parsed Rust file
pub(crate) fn find_const(parsed_file: &File, const_name: &str) -> Option<ItemConst> {
    let mut finder = ConstFinder::new(const_name);
    finder.visit_file(parsed_file);
    finder.const_item
}

/// A visitor that finds an enum by name
struct ConstFinder {
    const_name: String,
    const_item: Option<ItemConst>,
}

impl ConstFinder {
    pub fn new(const_name: &str) -> Self {
        Self {
            const_name: const_name.to_string(),
            const_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for ConstFinder {
    fn visit_item_const(&mut self, item_const: &'ast ItemConst) {
        if item_const.ident == self.const_name {
            self.const_item = Some(item_const.clone());
        }

        // Continue visiting
        visit::visit_item_const(self, item_const);
    }
}
