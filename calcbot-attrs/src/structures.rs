use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Attribute,
    Expr,
    Ident,
    ItemStruct,
    Lit,
    LitStr,
    Meta,
    Result,
    Token,
    Type,
    Visibility,
};

/// An array.
#[derive(Debug)]
pub struct Array<T> {
    pub elements: Vec<T>,
}

// the derive(Default) macro adds an implicit T: Default, which is unneeded (Vec<T>: Default)
impl<T> Default for Array<T> {
    fn default() -> Self {
        Array { elements: Vec::new() }
    }
}

impl<T: Parse> Parse for Array<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = Vec::new();

        let content;
        syn::bracketed!(content in input);

        while !content.is_empty() {
            result.push(content.parse()?);
            if content.parse::<Token![,]>().is_err() {
                break;
            }
        }

        Ok(Array { elements: result })
    }
}

/// A slice of static string literals.
#[derive(Debug)]
pub struct SliceLitStr(pub Array<LitStr>);

impl Parse for SliceLitStr {
    fn parse(input: ParseStream) -> Result<Self> {
        <Array<LitStr>>::parse(input).map(SliceLitStr)
    }
}

impl ToTokens for SliceLitStr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let inner = self.0.elements.iter().map(|lit| quote! { #lit, });
        tokens.extend(quote! { &[#(#inner)*] });
    }
}

/// The `children` argument of the `info` attribute. This tag consists of a [`Vec`] of
/// trait objects.
#[derive(Debug, Default)]
pub struct CommandGroup(pub Array<Type>);

impl Parse for CommandGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        <Array<Type>>::parse(input).map(CommandGroup)
    }
}

impl ToTokens for CommandGroup {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let inner = self.0.elements.iter().map(|lit| quote! { Box::new(#lit), });
        tokens.extend(quote! { crate::commands::CommandGroup::new(vec![#(#inner)*]) });
    }
}

/// The arguments that can be passed to the `info` attribute.
#[derive(Debug, Default)]
pub struct InfoArgs {
    pub category: Option<LitStr>,
    pub aliases: Option<SliceLitStr>,
    pub syntax: Option<SliceLitStr>,
    pub examples: Option<SliceLitStr>,
    pub children: CommandGroup,
}

impl InfoArgs {
    /// Parse the next argument in the input stream and applies it to itself.
    fn parse_arg(&mut self, input: ParseStream) -> Result<()> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![=]>()?;

        let ident_str = ident.to_string();
        match ident_str.as_str() {
            "category" => self.category = Some(input.parse()?),
            "aliases" => self.aliases = Some(input.parse()?),
            "syntax" => self.syntax = Some(input.parse()?),
            "examples" => self.examples = Some(input.parse()?),
            "children" => self.children = input.parse()?,
            _ => return Err(syn::Error::new_spanned(ident, format!("unknown tag `{}`", ident_str))),
        }

        Ok(())
    }
}

impl Parse for InfoArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = InfoArgs::default();

        while args.parse_arg(input).is_ok() {
            if input.parse::<Token![,]>().is_err() {
                break;
            }
        }

        Ok(args)
    }
}

#[derive(Debug)]
pub struct CommandInfo {
    pub name: Ident,
    pub description: String,
    pub info_args: InfoArgs,
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
        let mut info_args = InfoArgs::default();

        for attr in &attributes {
            let attr_name = attr.path().get_ident().unwrap();
            let ident = attr_name.to_string();
            match ident.as_str() {
                "doc" => {
                    let Meta::NameValue(meta) = attr.meta.clone() else {
                        return Err(syn::Error::new_spanned(attr, "doc comment should be a name-value pair"));
                    };
                    let Expr::Lit(expr_lit) = meta.value else {
                        return Err(syn::Error::new_spanned(meta, "doc comment should be a string literal"));
                    };
                    let Lit::Str(expr) = expr_lit.lit else {
                        return Err(syn::Error::new_spanned(expr_lit, "doc comment should be a string literal"));
                    };
                    let string = expr.value();
                    if string.is_empty() {
                        description.push_str("\n\n");
                    } else {
                        description.push_str(&string);
                    }
                },
                "info" => info_args = attr.parse_args::<InfoArgs>().unwrap(),
                _ => (), // TODO: may need to add back these ignored attributes
            }
        }

        Ok(CommandInfo {
            name,
            description,
            info_args,
        })
    }
}
