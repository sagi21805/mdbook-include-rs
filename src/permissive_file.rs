use proc_macro2::{Delimiter, LexError, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    Attribute, File, Item,
    parse::{Parse, ParseStream},
};

struct PermissiveFile {
    pub shebang: Option<String>,
    pub attrs: Vec<Attribute>,
    pub items: Vec<Item>,
}

impl Parse for PermissiveFile {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Mirror what syn does internally for File
        let shebang = parse_shebang(input);
        let attrs = input.call(Attribute::parse_inner)?;
        let mut items = Vec::new();

        while !input.is_empty() {
            let fork = input.fork();
            if fork.parse::<Item>().is_ok() {
                items.push(input.parse::<Item>()?);
                continue;
            }

            // Recovery: slurp tokens until the closing `}` or `;` of
            // the unrecognised item, then store as Item::Verbatim.
            let mut verbatim = TokenStream::new();
            loop {
                if input.is_empty() {
                    break;
                }
                let tt: TokenTree = input.parse()?;
                let done = matches!(
                    &tt,
                    TokenTree::Group(g) if g.delimiter() == Delimiter::Brace
                ) || matches!(
                    &tt,
                    TokenTree::Punct(p) if p.as_char() == ';'
                );
                tt.to_tokens(&mut verbatim);
                if done {
                    break;
                }
            }

            if !verbatim.is_empty() {
                items.push(Item::Verbatim(verbatim));
            }
        }

        Ok(PermissiveFile {
            shebang,
            attrs,
            items,
        })
    }
}

fn parse_shebang(input: ParseStream) -> Option<String> {
    // A shebang is `#!` at the very start of the file, not an attribute.
    // syn's own parse_file handles this; we replicate the minimal check.
    if input.peek(syn::token::Pound) {
        let fork = input.fork();
        // If it parses as an inner attribute it isn't a shebang
        if fork.call(Attribute::parse_inner).is_ok() {
            return None;
        }
    }
    None // proc_macro2 strips the shebang before we see the token stream
}

/// Drop-in replacement for `syn::parse_file` that doesn't hard-fail on
/// non-standard items like `const trait`.
pub fn parse_file_permissive(content: &str) -> syn::Result<File> {
    let ts: TokenStream = content
        .parse()
        .map_err(|e: LexError| syn::Error::new(proc_macro2::Span::call_site(), e.to_string()))?;

    let permissive = syn::parse2::<PermissiveFile>(ts)?;

    Ok(File {
        shebang: permissive.shebang,
        attrs: permissive.attrs,
        items: permissive.items,
    })
}
