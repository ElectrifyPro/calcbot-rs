mod structures;
mod util;

use proc_macro::TokenStream;
use quote::quote;
use structures::CommandInfo;
use syn::{parse_macro_input, ItemStruct};

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
/// | Tag           | Description                       | Accepts (types are converted automatically) | Obtained from                                                        |
/// |---------------|-----------------------------------|---------------------------------------------|----------------------------------------------------------------------|
/// | `name`        | The name of the command.          | `&str`                                      | The struct's name.                                                   |
/// | `description` | The description of the command.   | `&str`                                      | The struct's doc comment.                                            |
/// | `category`    | The category of the root command. | `&str`                                      | The `category` tag in the `info` attribute.                          |
/// | `aliases`     | Allowed aliases for the command.  | `[&str]`                                    | The struct's name, or via the `aliases` tag in the `info` attribute. |
/// | `syntax`      | The syntax of the command.        | `[&str]`                                    | The `syntax` tag in the `info` attribute.                            |
/// | `examples`    | Example usage of the command.     | `[&str]`                                    | The `examples` tag in the `info` attribute.                          |
/// | `children`    | The subcommands of the command.   | `[impl Command]`                            | The `children` tag in the `info` attribute.                          |
///
/// There are also some special tags that provide additional functionality:
///
/// ## `args`
///
/// The arguments of the command. If this tag is provided, a `parse_args` method will be added to
/// the scope. The function takes a slice of strings, and returns a tuple containing the requested
/// types. The types are specified as an array of types in the expected order.
///
/// The given types must be in scope, and must implement the [`std::str::FromStr`] trait. Note that
/// the `args` tag is separate from the `syntax` tag, and the syntax must be specified in the
/// `syntax` tag.
///
/// Optional arguments can be indicated by wrapping the type in an [`Option`].
///
/// If the generated parser is not sufficient, you can add the special `Unlimited` type to the end of
/// the array to indicate that the rest of the arguments should be treated as a single string. This
/// type should be placed at the end of the array.
///
/// ### Example
///
/// ```
/// #[info(args = [Option<u64>, u64, Unlimited], syntax = "[number] <number> <string>")]
/// pub struct Foo;
///
/// #[async_trait]
/// impl Command for Foo {
///     async fn execute(
///         &self,
///         _: Arc<State>,
///         _: Arc<Mutex<Database>>,
///         _: &Message,
///         _: &str,
///     ) -> Result<(), Box<dyn Error + Send + Sync>> {
///         let (a, b, c) = parse_args(args)?;
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_derive(Info, attributes(info))]
pub fn info(item: TokenStream) -> TokenStream {
    let info = parse_macro_input!(item as CommandInfo);
    let CommandInfo {
        name,
        description,
        info_args,
    } = info;

    let name_str = util::pascal_to_snake_case(&name.to_string());
    let description = description.trim();
    let category = util::wrap(info_args.category);
    let aliases = util::wrap(info_args.aliases);
    let syntax = util::wrap(info_args.syntax);
    let examples = util::wrap(info_args.examples);
    let children = info_args.children;

    let mut result = quote! {
        impl crate::commands::Info for #name {
            fn info(&self) -> crate::commands::CommandInfo {
                crate::commands::CommandInfo {
                    name: #name_str,
                    description: #description,
                    category: #category,
                    aliases: #aliases,
                    syntax: #syntax,
                    examples: #examples,
                    children: #children,
                }
            }
        }
    };

    if let Some(args) = info_args.args {
        result.extend(args.generate_parse_args());
    }

    quote! { #result }.into()
}

/// Derives the [`Command`] trait for the given struct. This is a convenience macro that
/// simply creates an empty implementation of the trait.
#[proc_macro_derive(Command)]
pub fn command(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStruct);
    let name = item.ident;
    quote! {
        #[async_trait::async_trait]
        impl crate::commands::Command for #name {
            async fn execute(
                &self,
                state: &std::sync::Arc<crate::global::State>,
                _: &std::sync::Arc<tokio::sync::Mutex<crate::database::Database>>,
                ctxt: &crate::commands::Context,
            ) -> Result<(), Box<dyn crate::error::Error + Send + Sync>> {
                // send the help embed by default
                let embed = self.info().build_embed(Some("c-"));
                state.http.create_message(ctxt.message.channel_id)
                    .embeds(&[embed])?
                    .await?;
                Ok(())
            }
        }
    }.into()
}
