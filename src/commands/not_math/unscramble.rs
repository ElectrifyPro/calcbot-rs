use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    /// The list of words to search through (~250K words).
    static ref WORDS: Vec<String> = {
        let words = std::fs::read_to_string("src/commands/not_math/words.json").unwrap();
        serde_json::from_str(&words).unwrap()
    };
}

/// Stack-allocated structure to hold English letter counts, as opposed to allocation
/// `HashMap<char, usize>` over and over for each word.
#[derive(Clone, Copy, PartialEq)]
struct LetterCounts {
    counts: [usize; 26],
}

impl LetterCounts {
    /// Counts the number of each English letter in the given string.
    fn count(s: &str) -> Self {
        let mut lc = Self { counts: [0; 26] };
        for c in s.chars() {
            lc.add(c);
        }
        lc
    }

    /// Adds a character to the counts.
    fn add(&mut self, c: char) {
        if c.is_ascii_alphabetic() {
            let idx = c.to_ascii_lowercase() as usize - 'a' as usize;
            self.counts[idx] += 1;
        }
    }

    /// Returns true if the word represented by `self` can be formed using letters from `other`.
    fn can_form_from(self, other: Self) -> bool {
        self.counts.iter()
            .zip(other.counts.iter())
            .all(|(a, b)| a <= b)
    }
}

/// Finds the words that can be spelt using the provided letters.
fn unscramble(letters: &str, length: usize) -> Vec<&'static str> {
    let mut words = Vec::new();
    let letters = LetterCounts::count(letters);

    for candidate in WORDS.iter() {
        if candidate.len() != length {
            continue;
        }

        let candidate_letters = LetterCounts::count(candidate);

        // the target word must have at least as many of each letter as the input
        if candidate_letters.can_form_from(letters) {
            words.push(candidate.as_str());
        }

        if words.len() >= 100 {
            break;
        }
    }

    words
}

/// _Don't use this to cheat at Anagrams._
///
/// Finds English words (up to 100) that can be spelt using the provided letters.
///
/// The command will only find words that use the same number of letters as your input. You can
/// change this by providing an optional second argument with the desired word length.
#[derive(Clone, Info)]
#[info(
    aliases = ["unscramble", "unsc", "uns"],
    syntax = ["<letters> [word length]"],
    examples = ["itonnnive", "aeht 3"],
    parent = super::NotMath,
)]
pub struct Unscramble;

#[async_trait]
impl Command for Unscramble {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(Word, Option<_>)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let word = parsed.0.0;
        let length = parsed.1.unwrap_or(word.len());

        let words = unscramble(word, length);
        let output = if words.is_empty() {
            "_no words found_".to_string()
        } else {
            words.join(", ")
        };

        ctxt.trigger.reply(&state.http)
            .content(&format!(
                "**Unscrambling** `{}` with word length of {}\n{}",
                word, length, output
            ))
            .await?;
        Ok(())
    }
}
