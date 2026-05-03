use proc_macro2::TokenStream;
use syn::{
    File, Item, ItemTrait,
    visit::{self, Visit},
};

use crate::const_trait::ItemTraitConst;

#[derive(Clone)]
pub enum TraitType {
    Const(TokenStream),
    Reg(ItemTrait),
}

impl TraitType {
    pub fn item(self) -> Item {
        match self {
            TraitType::Const(trait_item) => Item::Verbatim(trait_item),
            TraitType::Reg(trait_item) => Item::Trait(trait_item),
        }
    }
}

/// Find a trait in a parsed Rust file, handling both regular and const traits
pub fn find_trait(parsed_file: &File, trait_name: &str) -> Option<TraitType> {
    let mut finder = TraitFinder::new(trait_name);
    finder.visit_file(parsed_file);
    finder.trait_item
}

/// A visitor that finds a trait by name
pub struct TraitFinder {
    trait_name: String,
    trait_item: Option<TraitType>,
}

impl TraitFinder {
    pub fn new(trait_name: &str) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            trait_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for TraitFinder {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Trait(trait_item) => {
                if trait_item.ident == self.trait_name {
                    self.trait_item = Some(TraitType::Reg(trait_item.clone()));
                }
            }
            Item::Verbatim(tokens) => {
                if let Ok(trait_item) = syn::parse2::<ItemTraitConst>(tokens.clone()) {
                    if trait_item.ident == self.trait_name {
                        self.trait_item = Some(TraitType::Const(tokens.clone()));
                    }
                }
            }
            _ => {}
        }

        visit::visit_item(self, item);
    }
}
