pub mod units;

use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_math::unit_conversion::{Measurement, Unit};
use crate::{
    commands::{Command, Context},
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
        let raw_args = ctxt.raw_input.split_whitespace().collect::<Vec<_>>();
        let (quantity, unit, target_unit) = match raw_args.len() {
            3 => (raw_args[0].parse().unwrap(), Unit::try_from(raw_args[1]).unwrap(), Unit::try_from(raw_args[2]).unwrap()),
            _ => todo!(),
        };

        let start = Measurement::<f64>::new(quantity, unit);
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
