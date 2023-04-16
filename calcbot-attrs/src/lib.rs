use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    Attribute,
    Expr,
    Ident,
    ItemStruct,
    Lit,
    Meta,
    Result,
    Token,
    Visibility,
};

/// Converts an identifier in PascalCase to snake_case. (from `serde-derive`)
fn pascal_to_snake_case(name: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in name.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}

/// Wrap an [`Option<Token>`] in a token that actually wraps the token in an [`Option`].
///
/// This is different from the default implementation of [`ToTokens`] for [`Option`];
/// the default implementation never does any wrapping.
fn wrap<T: ToTokens>(option: Option<T>) -> TokenStream2 {
    match option {
        Some(token) => quote! { Some(#token) },
        None => quote! { None },
    }
}

/// A set of key-value pairs in an attribute, for example:
///
/// ```
/// #[info(aliases = ["foo", "bar"], syntax = "baz")]
/// //     ^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^ key-value pairs
/// ```
#[derive(Debug)]
struct KeyValuePairs {
    pairs: Vec<(Ident, Expr)>,
}

impl Parse for KeyValuePairs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut pairs = Vec::new();
        while !input.is_empty() {
            let ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let expr = input.parse()?;

            pairs.push((ident, expr));
            input.parse::<Token![,]>()?;
        }

        Ok(KeyValuePairs { pairs })
    }
}

#[derive(Debug)]
struct CommandInfo {
    pub name: Ident,
    pub description: String,
    pub aliases: Option<Expr>,
    pub syntax: Option<Expr>,
    pub examples: Option<Expr>,
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
        let mut aliases = None;
        let mut syntax = None;
        let mut examples = None;

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
                        match ident_name.as_str() {
                            "aliases" => aliases = Some(expr),
                            "syntax" => syntax = Some(expr),
                            "examples" => examples = Some(expr),
                            _ => panic!("unknown tag in #[info] attribute"),
                        }
                    }
                },
                _ => panic!("unknown attribute"),
            }
        }

        Ok(CommandInfo {
            name,
            description,
            aliases,
            syntax,
            examples,
        })
    }
}

/// Derives the [`Info`] trait for the given struct.
///
/// The information of the command can be customized using the `info` attribute by adding
/// the corresponding tags to it:
/// ```
/// #[info(aliases = ["foo", "bar"], syntax = "baz")]
/// pub struct Foo;
/// ```
///
/// This is where the macro gets its information from to implement the trait:
///
/// | Tag           | Description                      | Accepts   | Obtained from                                                        |
/// |---------------|----------------------------------|-----------|----------------------------------------------------------------------|
/// | `name`        | The name of the command.         | `&str`    | The struct's name.                                                   |
/// | `description` | The description of the command.  | `&str`    | The struct's doc comment.                                            |
/// | `aliases`     | Allowed aliases for the command. | `&[&str]` | The struct's name, or via the `aliases` tag in the `info` attribute. |
/// | `syntax`      | The syntax of the command.       | `&[&str]` | The `syntax` tag in the `info` attribute.                            |
/// | `examples`    | Example usage of the command.    | `&[&str]` | The `examples` tag in the `info` attribute.                          |
#[proc_macro_derive(Info, attributes(info))]
pub fn info(item: TokenStream) -> TokenStream {
    let info = parse_macro_input!(item as CommandInfo);
    let CommandInfo {
        name,
        description,
        aliases,
        syntax,
        examples,
    } = info;

    let name_str = pascal_to_snake_case(&name.to_string());
    let description = description.trim();
    let aliases = wrap(aliases);
    let syntax = wrap(syntax);
    let examples = wrap(examples);

    quote! {
        impl crate::commands::Info for #name {
            fn info(&self) -> crate::commands::CommandInfo {
                crate::commands::CommandInfo {
                    name: #name_str,
                    description: #description,
                    aliases: #aliases,
                    syntax: #syntax,
                    examples: #examples,
                }
            }
        }
    }.into()
}
