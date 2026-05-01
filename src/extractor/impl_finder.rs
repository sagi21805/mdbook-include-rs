use proc_macro2::{Span, TokenStream};
use syn::{
    File, Item, ItemImpl, Path, Type,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::const_impl::ItemImplConst;

#[derive(Clone)]
pub enum ImplType {
    Const(TokenStream),
    Reg(ItemImpl),
}

impl ImplType {
    pub fn item(self) -> Item {
        match self {
            ImplType::Const(impl_item) => Item::Verbatim(impl_item),
            ImplType::Reg(impl_item) => Item::Impl(impl_item),
        }
    }
}

/// Find a struct implementation in a parsed Rust file
pub(crate) fn find_struct_impl(parsed_file: &File, struct_name: &str) -> Option<ImplType> {
    let mut finder = StructImplFinder::new(struct_name);
    finder.visit_file(parsed_file);
    finder.impl_item
}

/// Find a trait implementation for a struct in a parsed Rust file
pub(crate) fn find_trait_impl(
    parsed_file: &File,
    trait_name: &str,
    struct_name: &str,
) -> (Vec<ImplType>, Vec<Span>) {
    let mut finder = TraitImplFinder::new(trait_name, struct_name);
    finder.visit_file(parsed_file);
    (finder.impl_items, finder.spans)
}

/// A visitor that finds a struct implementation by struct name
struct StructImplFinder {
    struct_name: String,
    impl_item: Option<ImplType>,
}

impl StructImplFinder {
    pub fn new(struct_name: &str) -> Self {
        Self {
            struct_name: struct_name.to_string(),
            impl_item: None,
        }
    }

    fn get_type_path<'a>(&self, ty: &'a Type) -> Option<&'a Path> {
        if let Type::Path(type_path) = ty {
            Some(&type_path.path)
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for StructImplFinder {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Impl(impl_item) => {
                if impl_item.trait_.is_none() {
                    if let Some(path) = self.get_type_path(&impl_item.self_ty) {
                        if path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == self.struct_name)
                        {
                            self.impl_item = Some(ImplType::Reg(impl_item.clone()));
                        }
                    }
                }
            }
            Item::Verbatim(tokens) => {
                let const_impl = syn::parse2::<ItemImplConst>(tokens.clone());
                if let Ok(impl_item) = const_impl {
                    if impl_item.trait_.is_none() {
                        if let Some(path) = self.get_type_path(&impl_item.self_ty) {
                            if path
                                .segments
                                .last()
                                .is_some_and(|seg| seg.ident == self.struct_name)
                            {
                                self.impl_item = Some(ImplType::Const(tokens.clone()));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        visit::visit_item(self, item);
    }
}

/// A visitor that finds a trait implementation for a struct
pub struct TraitImplFinder {
    trait_name: String,
    struct_name: String,
    impl_items: Vec<ImplType>,
    spans: Vec<Span>,
}

impl TraitImplFinder {
    pub fn new(trait_name: &str, struct_name: &str) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            struct_name: struct_name.to_string(),
            impl_items: Vec::new(),
            spans: Vec::new(),
        }
    }

    fn get_type_path<'a>(&self, ty: &'a Type) -> Option<&'a Path> {
        if let Type::Path(type_path) = ty {
            Some(&type_path.path)
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for TraitImplFinder {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Impl(impl_item) => {
                if let Some((_, trait_path, _)) = &impl_item.trait_ {
                    if trait_path
                        .segments
                        .last()
                        .is_some_and(|seg| seg.ident == self.trait_name)
                    {
                        if let Some(path) = self.get_type_path(&impl_item.self_ty) {
                            if path
                                .segments
                                .last()
                                .is_some_and(|seg| seg.ident == self.struct_name)
                            {
                                self.impl_items.push(ImplType::Reg(impl_item.clone()));
                                self.spans.push(impl_item.span());
                            }
                        }
                    }
                }
            }
            Item::Verbatim(tokens) => {
                let const_impl = syn::parse2::<ItemImplConst>(tokens.clone());
                if let Ok(impl_item) = const_impl {
                    if let Some((_, trait_path, _)) = &impl_item.trait_ {
                        if trait_path
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == self.trait_name)
                        {
                            if let Some(path) = self.get_type_path(&impl_item.self_ty) {
                                if path
                                    .segments
                                    .last()
                                    .is_some_and(|seg| seg.ident == self.struct_name)
                                {
                                    self.impl_items.push(ImplType::Const(tokens.clone()));
                                    self.spans.push(tokens.span());
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        visit::visit_item(self, item);
    }
}

/// Find a specific method inside a struct's inherent impl block.
pub(crate) fn find_impl_methods(
    parsed_file: &File,
    struct_name: &str,
    methods: Vec<&str>,
) -> (Vec<syn::ImplItemFn>, Vec<Span>) {
    let mut finder = ImplMethodFinder::new(struct_name, methods);
    finder.visit_file(parsed_file);
    (finder.functions, finder.spans)
}

struct ImplMethodFinder<'a> {
    struct_name: String,
    methods: Vec<&'a str>,
    functions: Vec<syn::ImplItemFn>,
    spans: Vec<Span>,
}

impl<'a> ImplMethodFinder<'a> {
    fn new(struct_name: &'a str, methods: Vec<&'a str>) -> Self {
        Self {
            struct_name: struct_name.to_string(),
            methods,
            functions: Vec::new(),
            spans: Vec::new(),
        }
    }
}

impl<'a, 'ast> Visit<'ast> for ImplMethodFinder<'a> {
    fn visit_item(&mut self, item: &'ast Item) {
        match item {
            Item::Impl(impl_item) => {
                if impl_item.trait_.is_none() {
                    if let Type::Path(tp) = &*impl_item.self_ty {
                        if tp
                            .path
                            .segments
                            .last()
                            .is_some_and(|s| s.ident == self.struct_name)
                        {
                            let impl_header_span = impl_item
                                .impl_token
                                .span()
                                .join(impl_item.brace_token.span.open())
                                .unwrap_or_else(|| impl_item.impl_token.span());
                            self.spans.push(impl_header_span);
                            for impl_item in &impl_item.items {
                                if let syn::ImplItem::Fn(method) = impl_item {
                                    if self
                                        .methods
                                        .contains(&method.sig.ident.to_string().as_str())
                                    {
                                        self.functions.push(method.clone());
                                        self.spans.push(method.span());
                                    }
                                }
                            }
                            self.spans.push(impl_item.brace_token.span.close());
                        }
                    }
                }
            }
            Item::Verbatim(tokens) => {
                let const_impl = syn::parse2::<ItemImplConst>(tokens.clone());
                if let Ok(impl_item) = const_impl {
                    if impl_item.trait_.is_none() {
                        if let Type::Path(tp) = &*impl_item.self_ty {
                            if tp
                                .path
                                .segments
                                .last()
                                .is_some_and(|s| s.ident == self.struct_name)
                            {
                                let impl_header_span = impl_item
                                    .impl_token
                                    .span()
                                    .join(impl_item.brace_token.span.open())
                                    .unwrap_or_else(|| impl_item.impl_token.span());
                                self.spans.push(impl_header_span);
                                for item in &impl_item.items {
                                    if let syn::ImplItem::Fn(method) = item {
                                        if self
                                            .methods
                                            .contains(&method.sig.ident.to_string().as_str())
                                        {
                                            self.functions.push(method.clone());
                                            self.spans.push(method.span());
                                        }
                                    }
                                }
                                self.spans.push(impl_item.brace_token.span.close());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        visit::visit_item(self, item);
    }
}
