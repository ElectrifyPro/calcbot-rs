use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Sorts a list of numbers / words in ascending / alphabetical order, numbers first. If a minus
/// symbol (`-`) is provided for the first argument, the list will be sorted in descending order
/// instead.
#[derive(Clone, Info)]
#[info(
    aliases = ["sort", "so"],
    syntax = ["<number | word>...", "- <number | word>..."],
    examples = ["9 5 28 12", "5 11 this sentence is not in alphabetical order 2"],
)]
pub struct Sort;

#[async_trait]
impl Command for Sort {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut args = ctxt.raw_input.split_whitespace().collect::<Vec<_>>();
        let descending = args[0] == "-";
        if descending {
            args.remove(0);
        }

        let mut numbers = Vec::new();
        let mut words = Vec::new();
        for arg in args {
            if let Ok(number) = arg.parse::<f64>() {
                numbers.push(number);
            } else {
                words.push(arg);
            }
        }

        let mut output = String::new();
        if descending {
            output.push_str("**Sorting** in descending order\n");
            numbers.sort_by(|a, b| b.partial_cmp(a).unwrap());
            words.sort_by(|a, b| b.partial_cmp(a).unwrap());
        } else {
            output.push_str("**Sorting** in ascending order\n");
            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());
            words.sort();
        }

        let values = numbers
            .into_iter()
            .map(|n| n.to_string())
            .chain(words.into_iter().map(|w| w.to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&values);

        ctxt.trigger.reply(&state.http)
            .content(&output)?
            .await?;
        Ok(())
    }
}
