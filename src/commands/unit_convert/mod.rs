pub mod units;

use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_unit_convert::{Measurement, Unit};
use crate::{
    arg_parse::{Word, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Convert a quantity from one unit / ratio to another.
///
/// CalcBot supports server-unique and user-unique custom ratios; run `{prefix}unitconvert
/// customratio` for more info. Run `{prefix}unitconvert units` to see a list of supported units.
///
/// **CalcBot uses the US customary measurement system.** You can read about its differences with
/// the imperial system
/// [here](https://en.wikipedia.org/wiki/Comparison_of_the_imperial_and_US_customary_measurement_systems).
#[derive(Clone, Info)]
#[info(
    category = "Calculate",
    aliases = ["unitconvert", "uc"],
    syntax = ["<quantity> <unit / ratio> <target unit / ratio>"],
    examples = ["18 sec min", "14 mi/hr km/sec"],
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
        let parsed = parse_args_full::<(f64, Word, Word)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument | Error::TooManyArguments) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;
        let quantity = parsed.0;

        let prefix = ctxt.prefix.unwrap_or_default();
        let Ok(unit) = parsed.1.0.parse::<Unit>() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{}` is not a valid unit.** Run the `{prefix}unitconvert units` for a list of supported units.", parsed.1.0))?
                .await?;
            return Ok(());
        };
        let Ok(target_unit) = parsed.2.0.parse::<Unit>() else {
            ctxt.trigger.reply(&state.http)
                .content(&format!("**`{}` is not a valid unit.** Run the `{prefix}unitconvert units` for a list of supported units.", parsed.2.0))?
                .await?;
            return Ok(());
        };

        let start = Measurement::new(quantity, unit);
        let out_msg = match start.convert(target_unit) {
            Ok(end) => {
                format!("**Converting** `{} {}` to `{}`\n{}", quantity, unit, target_unit, end.value())
            },
            Err(_) => {
                format!("**Can't convert from from `{}` to `{}`.**", unit, target_unit)
            },
        };
        ctxt.trigger.reply(&state.http)
            .content(&out_msg)?
            .await?;

        Ok(())
    }
}
