use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::{CustomErrorFmt, Error},
    global::State,
};
use reqwest::get;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use twilight_http::{
    request::channel::message::CreateMessage,
    response::ResponseFuture,
};
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};
use twilight_validate::message::MessageValidationError;

/// A language code supported by the Google Dictionary API.
type LanguageCode<'a> = &'a str;

/// All languages codes supported by the Google Dictionary API.
const LANGUAGES: [LanguageCode<'static>; 12] = [
    "en", "hi", "es", "fr", "ru", "de", "it", "ko", "pt-BR", "zh-CN", "ar", "tr",
];

const SUPERSCRIPT_NUMBERS: [&str; 10] = ["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];

/// Returns the given number in superscript.
fn fmt_superscript(number: usize) -> String {
    number
        .to_string()
        .chars()
        .map(|digit| SUPERSCRIPT_NUMBERS[digit as usize - '0' as usize])
        .collect::<String>()
}

/// Represents a semantic domain of a word or phrase, a grouping of related meanings for a word.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Domain {
    /// The word or phrase.
    word: String,

    /// The meanings of the word or phrase.
    meanings: Vec<Meaning>,
}

/// Represents a specific meaning of a word or phrase.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Meaning {
    /// The part of speech of the word or phrase.
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,

    /// The definitions of the word or phrase.
    definitions: Vec<Definition>,

    /// The synonyms of the word or phrase. This can be empty.
    synonyms: Vec<String>,

    /// The antonyms of the word or phrase. This can be empty.
    antonyms: Vec<String>,
}

/// Represents a specific definition of a word or phrase.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Definition {
    /// The definition of the word or phrase.
    definition: String,

    /// Synonyms of the specific definition. This can be empty.
    synonyms: Vec<String>,

    /// Antonyms of the specific definition. This can be empty.
    antonyms: Vec<String>,

    /// The example of the word or phrase.
    example: Option<String>,
}

lazy_static::lazy_static! {
    /// Cache of words that have already been searched for.
    static ref CACHE: HashMap<LanguageCode<'static>, Vec<Domain>> = HashMap::new();
}

/// Generic error type for we failed to fetch a word or phrase from the Google Dictionary API.
#[derive(Debug)]
enum FetchError {
    /// The language code was invalid.
    InvalidLanguageCode(String),

    /// The word or phrase was not found in the given language.
    NotFound(String, String),

    /// An error occurred while fetching the word or phrase.
    Reqwest,
}

impl CustomErrorFmt for FetchError {
    fn rich_fmt(&self, init: CreateMessage<'_>) -> Result<ResponseFuture<Message>, MessageValidationError> {
        match self {
            FetchError::InvalidLanguageCode(language) => Ok(init.content(&format!("**The language code `{}` is invalid.** See [this link](<https://chillant.gitbook.io/calcbot/commands/dictionary>) for a list of valid language codes.", language)).into_future()),
            FetchError::NotFound(word, language) => Ok(init.content(&format!("**Could not find a dictionary entry for `{}` in the `{}` dictionary.**", word, language)).into_future()),
            FetchError::Reqwest => Ok(init.content("**An error occurred while fetching the definition. Please try again in a few seconds.**").into_future()),
        }
    }
}

/// Fetch the Google Dictionary entry of a word or phrase, using the cache if possible.
async fn get_dictionary_entry<'a>(
    word: &'a str,
    language: LanguageCode<'a>,
) -> Result<Vec<Domain>, FetchError> {
    if !LANGUAGES.contains(&language) {
        return Err(FetchError::InvalidLanguageCode(language.to_string()));
    }

    if let Some(entry) = CACHE.get(language).and_then(|domains| {
        domains
            .iter()
            .find(|domain| domain.word.to_lowercase() == word.to_lowercase())
    }) {
        return Ok(vec![entry.clone()]);
    }

    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/{}/{}",
        language, word
    );
    let response = get(&url)
        .await
        .map_err(|_| FetchError::Reqwest)?
        .json::<Vec<Domain>>()
        .await
        .map_err(|_| FetchError::NotFound(word.to_string(), language.to_string()))?;
    Ok(response)
}

/// Get the Google Dictionary entry of a word or phrase. You may also provide a [language
/// code](https://chillant.gitbook.io/calcbot/commands/dictionary) for the second argument to
/// search that language's dictionary.
///
/// **Note: CalcBot will not filter profanity or other words that might be considered offensive.
/// Use with caution.**
#[derive(Clone, Info)]
#[info(
    category = "Text",
    aliases = ["dictionary", "define", "dict", "def"],
    syntax = ["<word | phrase> [language code]"],
    examples = ["hello", "안녕 ko"],
)]
pub struct Dictionary;

#[async_trait]
impl Command for Dictionary {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(Word, Option<Word>)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let word = parsed.0.0;
        let language = parsed.1
            .map(|lang| lang.0.to_ascii_lowercase())
            .unwrap_or_else(|| "en".to_string());

        let entries = get_dictionary_entry(word, &language).await?;
        let mut embed = EmbedBuilder::new()
            .title(word)
            .color(0x3468eb);

        for (superscript, domain) in entries.into_iter().enumerate() {
            let superscript = fmt_superscript(superscript + 1);
            for meaning in domain.meanings {
                let mut description = Vec::new();
                let mut synonym_antonyms = String::new();
                if !meaning.synonyms.is_empty() {
                    synonym_antonyms
                        .push_str(&format!("**Synonyms**: {}", meaning.synonyms.join(", ")));
                }

                if !meaning.antonyms.is_empty() {
                    synonym_antonyms
                        .push_str(&format!("**Antonyms**: {}", meaning.antonyms.join(", ")));
                }

                if !synonym_antonyms.is_empty() {
                    description.push(synonym_antonyms);
                }

                for definition in meaning.definitions {
                    let mut definition_parts = Vec::new();
                    definition_parts.push(definition.definition);
                    if let Some(example) = definition.example {
                        definition_parts.push(format!("_{}_", example));
                    }
                    if !definition.synonyms.is_empty() {
                        definition_parts
                            .push(format!("**Synonyms**: {}", definition.synonyms.join(", ")));
                    }
                    if !definition.antonyms.is_empty() {
                        definition_parts
                            .push(format!("**Antonyms**: {}", definition.antonyms.join(", ")));
                    }
                    description.push(definition_parts.join("\n"));

                    // embed fields have a 1024 character limit
                    // description.len() * 22 is the number of characters used for dividers
                    if description.iter().fold(0, |acc, x| acc + x.len()) + description.len() * 22 > 1024 {
                        description.pop();
                        break;
                    }
                }

                embed = embed.field(
                    EmbedFieldBuilder::new(
                        format!("{}{}", &meaning.part_of_speech, superscript),
                        description.join("\n**――――――――――――――――**\n"),
                    )
                    .inline(),
                );
            }
        }

        ctxt.trigger.reply(&state.http)
            .embeds(&[embed.build()])
            .await?;

        Ok(())
    }
}
