use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use super::util::validate_expr_array;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    Attribute,
    Expr,
    ExprLit,
    Ident,
    ItemStruct,
    Lit,
    Meta,
    Result,
    Token,
    Visibility,
};

/// A set of key-value pairs in an attribute, where the keys and values are separated by `=`. For
/// example:
///
/// ```
/// #[info(aliases = ["foo", "bar"], syntax = ["baz"])]
/// //     ^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^ key-value pairs
/// ```
#[derive(Debug)]
pub struct KeyValuePairs {
    pub pairs: Vec<(Ident, Expr)>,
}

impl Parse for KeyValuePairs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pairs = Vec::new();
        while !input.is_empty() {
            let ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let expr = input.parse()?;

            pairs.push((ident, expr));
            if input.parse::<Token![,]>().is_err() {
                break;
            }
        }

        Ok(KeyValuePairs { pairs })
    }
}

#[derive(Debug)]
pub struct CommandInfo {
    pub name: Ident,
    pub description: String,
    pub category: Option<TokenStream2>,
    pub aliases: Option<TokenStream2>,
    pub syntax: Option<TokenStream2>,
    pub examples: Option<TokenStream2>,
    pub children: Option<TokenStream2>,
}

impl Parse for CommandInfo {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse outer attributes, including documentation and `info` attributes
        let attributes = input.call(Attribute::parse_outer)?;

        // "pub struct ..."
        input.parse::<Visibility>().ok();
        let remaining = input.parse::<ItemStruct>()?;

        let name = remaining.ident;
        let mut description = String::new();
        let mut category = None;
        let mut aliases = None;
        let mut syntax = None;
        let mut examples = None;
        let mut children = None;

        for attr in &attributes {
            let attr_name = attr.path().get_ident().unwrap();
            let ident = attr_name.to_string();
            match ident.as_str() {
                "doc" => {
                    let Meta::NameValue(meta) = attr.meta.clone() else {
                        panic!("doc comment should be a name-value pair");
                    };
                    let Expr::Lit(expr_lit) = meta.value else {
                        panic!("doc comment should be a literal");
                    };
                    let Lit::Str(expr) = expr_lit.lit else {
                        panic!("doc comment should be a string literal");
                    };
                    let string = expr.value();
                    if string.is_empty() {
                        description.push_str("\n\n");
                    } else {
                        description.push_str(&string);
                    }
                },
                "info" => {
                    let KeyValuePairs { pairs } = attr.parse_args()?;
                    for (ident, expr) in pairs {
                        let ident_name = ident.to_string();
                        if ident_name.as_str() == "category" {
                            let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = expr else {
                                return Err(syn::Error::new_spanned(expr, "expected a string literal here"));
                            };
                            category = Some(quote! { #lit_str });
                        } else {
                            let mut expr_array = validate_expr_array(expr)?;
                            match ident_name.as_str() {
                                "aliases" => aliases = Some(quote! { &#expr_array }),
                                "syntax" => syntax = Some(quote! { &#expr_array }),
                                "examples" => examples = Some(quote! { &#expr_array }),
                                "children" => {
                                    expr_array.elems.iter_mut().for_each(|elem| {
                                        *elem = parse_quote! { Box::new(#elem) as Box<dyn crate::commands::Command> }
                                    });
                                    children = Some(quote! { crate::commands::CommandGroup::new(vec!#expr_array) });
                                },
                                _ => return Err(syn::Error::new_spanned(ident, format!("unknown attribute `{}` in `info`", ident_name))),
                            }
                        }
                    }
                },
                _ => (), // TODO: may need to add back these ignored attributes
            }
        }

        Ok(CommandInfo {
            name,
            description,
            category,
            aliases,
            syntax,
            examples,
            children: children.or(Some(quote! { crate::commands::CommandGroup::new(vec![]) })),
        })
    }
}
