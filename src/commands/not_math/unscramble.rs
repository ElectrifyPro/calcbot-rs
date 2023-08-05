use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    /// The list of words to search through (~250K words).
    static ref WORDS: Vec<&'static str> = {
        let words = include_str!("./words.json");
        serde_json::from_str(words).unwrap()
    };
}

/// Count the number of times each letter appears in a string.
fn count_letters(string: &str) -> HashMap<char, usize> {
    let mut letters = HashMap::new();

    for letter in string.to_lowercase().chars() {
        if letter.is_alphabetic() {
            *letters.entry(letter).or_insert(0) += 1;
        }
    }

    letters
}

/// Finds the words that can be spelt using the provided letters.
fn unscramble(letters: &str, length: usize) -> Vec<&'static str> {
    let mut words = Vec::new();
    let letters = count_letters(letters);

    for candidate in WORDS.iter() {
        if candidate.len() != length {
            continue;
        }

        let candidate_letters = count_letters(candidate);

        // the target word must have at least as many of each letter as the input
        if candidate_letters
            .iter()
            .all(|(letter, count)| letters.get(letter).map(|c| c >= count).unwrap_or(false))
        {
            words.push(*candidate);
        }

        if words.len() >= 100 {
            break;
        }
    }

    words
}

/// Finds words (up to 100) that can be spelt using the provided letters. The length of the input is used as the word length if not provided.
#[derive(Clone, Info)]
#[info(
    aliases = ["unscramble", "unsc", "uns"],
    syntax = ["<word> [word length]"],
    examples = ["itonnnive"],
    args = [&str, Option<usize>],
)]
pub struct Unscramble;

#[async_trait]
impl Command for Unscramble {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (word, length) = parse_args(ctxt.raw_input.split_whitespace().collect::<Vec<_>>())?;
        let length = length.unwrap_or(word.len());

        let words = unscramble(word, length);
        let output = if words.is_empty() {
            "_no words found_".to_string()
        } else {
            words.join(", ")
        };

        state.http.create_message(ctxt.message.channel_id)
            .content(&format!(
                "**Unscrambling** `{}` with word length of {}\n{}",
                word, length, output
            ))?
            .await?;
        Ok(())
    }
}
