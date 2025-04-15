use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    /// Words to ignore when converting to title case.
    static ref IGNORE: HashSet<&'static str> = [
        "a", "aboard", "about", "above", "across", "after", "against", "along", "amid", "among", "an", "and", "around", "as", "at", "before", "behind", "below", "beneath", "beside", "between", "beyond", "but", "by", "concerning", "considering", "despite", "down", "during", "except", "following", "for", "from", "in", "inside", "into", "like", "minus", "near", "next", "nor", "of", "off", "on", "onto", "opposite", "or", "out", "outside", "over", "past", "per", "plus", "regarding", "round", "save", "since", "so", "than", "the", "through", "to", "toward", "under", "underneath", "unlike", "until", "up", "upon", "versus", "via", "with", "within", "without", "yet",
    ].into_iter().collect();
}

/// Capitalizes the first letter of a string.
fn capitalize(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Converts text to title case. Prepositions and similar words will automatically be ignored.
#[derive(Clone, Info)]
#[info(
    aliases = ["title", "t"],
    syntax = ["<string>"],
    examples = ["the great escape", "what you should do in the event of apocalypse"],
    parent = super::NotMath,
)]
pub struct Title;

#[async_trait]
impl Command for Title {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let content = ctxt.raw_input
            .split_whitespace()
            .enumerate()
            .map(|(i, word)| {
                let lowercase = word.to_lowercase();
                if i == 0 || word.len() >= 5 || !IGNORE.contains(&lowercase.as_str()) {
                    capitalize(word)
                } else {
                    lowercase
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        ctxt.trigger.reply(&state.http)
            .content(&content)?
            .await?;
        Ok(())
    }
}
