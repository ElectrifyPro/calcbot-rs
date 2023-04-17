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
/// | `children`    | The subcommands of the command.   | `[dyn Command]`                             | The `children` tag in the `info` attribute.                          |
#[proc_macro_derive(Info, attributes(info))]
pub fn info(item: TokenStream) -> TokenStream {
    let info = parse_macro_input!(item as CommandInfo);
    let CommandInfo {
        name,
        description,
        category,
        aliases,
        syntax,
        examples,
        children,
    } = info;

    let name_str = util::pascal_to_snake_case(&name.to_string());
    let description = description.trim();
    let category = util::wrap(category);
    let aliases = util::wrap(aliases);
    let syntax = util::wrap(syntax);
    let examples = util::wrap(examples);

    quote! {
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
    }.into()
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
                state: Arc<crate::global::State>,
                message: &twilight_model::channel::message::Message,
                _: Vec<&str>,
            ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                // send the help embed by default
                let embed = self.info().build_embed(Some("c-"));
                state.http.create_message(message.channel_id)
                    .embeds(&[embed])?
                    .await?;
                Ok(())
            }
        }
    }.into()
}
