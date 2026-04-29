use syn::{
    File, ItemStatic,
    visit::{self, Visit},
};

/// Find an enum in a parsed Rust file
pub(crate) fn find_static(parsed_file: &File, static_name: &str) -> Option<ItemStatic> {
    let mut finder = StaticFinder::new(static_name);
    finder.visit_file(parsed_file);
    finder.static_item
}

/// A visitor that finds an enum by name
struct StaticFinder {
    static_name: String,
    static_item: Option<ItemStatic>,
}

impl StaticFinder {
    pub fn new(static_name: &str) -> Self {
        Self {
            static_name: static_name.to_string(),
            static_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for StaticFinder {
    fn visit_item_static(&mut self, item_static: &'ast ItemStatic) {
        if item_static.ident == self.static_name {
            self.static_item = Some(item_static.clone());
        }

        // Continue visiting
        visit::visit_item_static(self, item_static);
    }
}
