pub mod units;

use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_unit_convert::{CompoundUnit, Measurement};
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use itertools::{Either, Itertools};
use literator::Literator;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Convert a quantity from one unit to another. You can specify multiple target units to convert
/// to (see examples).
///
/// **CalcBot uses the US customary measurement system.** You can read about its differences with
/// the imperial system
/// [here](https://en.wikipedia.org/wiki/Comparison_of_the_imperial_and_US_customary_measurement_systems).
#[derive(Clone, Info)]
#[info(
    category = "Calculate",
    aliases = ["unitconvert", "uc"],
    syntax = ["<quantity> <unit> <target unit...>"],
    examples = ["18 s min", "14 mi/hr km/s", "600 min s hr day", "25 kg*m^2 lb*ft^2"],
    children = [units::Units],
)]
pub struct UnitConvert;

#[async_trait]
impl Command for UnitConvert {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>, // TODO: custom ratios
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let parsed = parse_args_full::<(f64, Word, Vec<Word>)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let quantity = parsed.0;

        let prefix = ctxt.prefix.unwrap_or_default();
        let Ok(unit) = parsed.1.0.parse::<CompoundUnit>() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{}` is not a valid unit.** Run the `{prefix}unitconvert units` for a list of supported units.", parsed.1.0))?
                .await?;
            return Ok(());
        };
        let unit_formatted = format!("{}", unit);

        let (target_units, errors): (Vec<_>, Vec<_>) = parsed.2
            .into_iter()
            .partition_map(|word| {
                match word.0.parse::<CompoundUnit>() {
                    Ok(unit) => Either::Left(unit),
                    Err(_) => Either::Right(word.0),
                }
            });
        if !errors.is_empty() {
            let msg = if errors.len() == 1 {
                format!("**`{}` is not a valid unit.** Run the `{prefix}unitconvert units` for a list of supported units.", errors[0])
            } else {
                let out = errors
                    .into_iter()
                    .oxford_join_custom("`, `", "` and `", "`, and `");
                format!("**`{}` are not valid units.** Run the `{prefix}unitconvert units` for a list of supported units.", out)
            };
            ctxt.trigger.reply(&state.http).content(&msg)?.await?;
            return Ok(());
        }

        let start = Measurement::new(quantity, unit);
        let conversions = target_units
            .into_iter()
            .map(|target_unit| start.convert(target_unit))
            .collect::<Result<Vec<_>, _>>();
        let out_msg = match conversions {
            Ok(conversions) => {
                let targets = conversions
                    .iter()
                    .map(|m| format!("`{}`", m.unit()))
                    .oxford_join_custom("`, `", "` and `", "`, and `");
                let results = conversions
                    .iter()
                    .map(|m| format!("`{} {}`", m.value(), m.unit()))
                    .join("\n");
                format!("**Converting** `{} {}` to `{}`\n{}", quantity, unit_formatted, targets, results)
            },
            Err(error) => {
                format!("**Can't convert from from `{}` to `{}`.**", unit_formatted, error.target())
            },
        };
        ctxt.trigger.reply(&state.http)
            .content(&out_msg)?
            .await?;

        Ok(())
    }
}
