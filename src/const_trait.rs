use syn::{
    Attribute, Generics, Ident, ItemTrait, Token, TraitItem, TypeParamBound, Visibility,
    WhereClause,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

#[derive(Clone, Debug)]
pub struct ItemTraitConst {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub unsafety: Option<Token![unsafe]>,
    pub auto_token: Option<Token![auto]>,
    pub constness: Option<Token![const]>,
    pub trait_token: Token![trait],
    pub ident: Ident,
    pub generics: Generics,
    pub colon_token: Option<Token![:]>,
    pub supertraits: Punctuated<TypeParamBound, Token![+]>,
    pub brace_token: token::Brace,
    pub items: Vec<TraitItem>,
}

impl Parse for ItemTraitConst {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;

        let unsafety: Option<Token![unsafe]> = if input.peek(Token![unsafe]) {
            Some(input.parse()?)
        } else {
            None
        };

        let auto_token: Option<Token![auto]> = if input.peek(Token![auto]) {
            Some(input.parse()?)
        } else {
            None
        };

        let constness: Option<Token![const]> = if input.peek(Token![const]) {
            Some(input.parse()?)
        } else {
            None
        };

        let trait_token: Token![trait] = input.parse()?;
        let ident: Ident = input.parse()?;
        let mut generics: Generics = input.parse()?;

        let colon_token: Option<Token![:]>;
        let mut supertraits: Punctuated<TypeParamBound, Token![+]> = Punctuated::new();

        if input.peek(Token![:]) {
            colon_token = Some(input.parse()?);
            loop {
                // Stop when we hit a where clause or the opening brace
                if input.peek(Token![where]) || input.peek(token::Brace) {
                    break;
                }
                supertraits.push_value(input.parse::<TypeParamBound>()?);
                if input.peek(Token![where]) || input.peek(token::Brace) {
                    break;
                }
                supertraits.push_punct(input.parse::<Token![+]>()?);
            }
        } else {
            colon_token = None;
        }

        generics.where_clause = input.parse::<Option<WhereClause>>()?;

        let content;
        let brace_token = syn::braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse::<TraitItem>()?);
        }

        Ok(ItemTraitConst {
            attrs,
            vis,
            unsafety,
            auto_token,
            constness,
            trait_token,
            ident,
            generics,
            colon_token,
            supertraits,
            brace_token,
            items,
        })
    }
}
