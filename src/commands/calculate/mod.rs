pub mod list_definitions;
pub mod mode;
pub mod to_latex;

use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_error::{Error as CasError};
use cas_parser::parser::Parser;
use cas_vm::Vm;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use strip_ansi_escapes::strip;
use std::sync::Arc;
use tokio::sync::Mutex;

fn report_errors(input: &str, errs: impl IntoIterator<Item = CasError>) -> String {
    errs.into_iter()
        .map(|err| {
            let mut buf = Vec::new();
            err.build_report("input")
                .write(("input", Source::from(input)), &mut buf)
                .unwrap();
            String::from_utf8(strip(buf).unwrap()).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Calculates an expression, like `1 + 1`.
///
/// You can create **variables** to save values for later use. For example, running
/// `{prefix}calculate x = 5` will save the value `5` to the variable `x`, which you can reference
/// in later calculations.
///
/// You can also create **functions** to save _entire calculations_. Running `{prefix}calculate
/// f(x) = x^2 + 5x + 6` saves the calculation `x^2 + 5x + 6` to the function `f`. You can then
/// repeat this calculation with different values for `x` by running `{prefix}calculate f(2)`,
/// `{prefix}calculate f(10)`, etc.
///
/// There is much more you can do! Learn more about this command here:
/// <https://chillant.gitbook.io/calcbot/calcbot-development-preview/cas>
#[derive(Clone, Info)]
#[info(
    category = "Calculate",
    aliases = ["calculate", "calc", "c"],
    syntax = ["<expression>"],
    examples = ["1+1", "x=2", "5sin(pi/2)", "6!", "f(x)=x^2+5x+6", "f(2)", "cos'(0)"],
    children = [
        list_definitions::ListDefinitions,
        mode::Mode,
        to_latex::ToLatex,
    ],
)]
pub struct Calculate;

#[async_trait]
impl Command for Calculate {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let mut parser = Parser::new(ctxt.raw_input);
        match parser.try_parse_full_many() {
            Ok(stmts) => {
                // let mut database = database.lock().await;
                // let eval_ctxt = database.get_user_field_mut::<Ctxt>(ctxt.trigger.author_id()).await;

                let mut vm = match Vm::compile_program(stmts) {
                    Ok(vm) => vm,
                    Err(err) => {
                        ctxt.trigger.reply(&state.http)
                            .content(&format!("```rs\n{}\n```", report_errors(ctxt.raw_input, Some(err))))
                            .await?;
                        return Ok(());
                    },
                };
                let cancel = vm.stop_execution.clone();
                tokio::select! {
                    _ = tokio::time::sleep(std::time::Duration::from_secs(4)) => {
                        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                        ctxt.trigger.reply(&state.http)
                            .content("**Timeout: The calculation took too long (4+ seconds) and was cancelled.** Check your expression to see if there are any mistakes or infinite loops.")
                            .await?;
                    }
                    Ok(out) = tokio::task::spawn_blocking(move || vm.run()) => {
                        let ans = match out {
                            Ok(ans) => ans,
                            Err(err) => {
                                ctxt.trigger.reply(&state.http)
                                    .content(&format!("```rs\n{}\n```", report_errors(ctxt.raw_input, Some(err))))
                                    .await?;
                                return Ok(());
                            },
                        };
                        ctxt.trigger.reply(&state.http)
                            .content(&format!("**Calculation**\n{}", ans))
                            // .content(&format!("**Calculation** (mode: {})\n{}", eval_ctxt.trig_mode, ans))
                            .await?;

                        // eval_ctxt.add_var("ans", ans);
                        // database.commit_user_field::<Ctxt>(ctxt.trigger.author_id()).await;
                    }
                }
            },
            Err(errs) => {
                ctxt.trigger.reply(&state.http)
                    .content(&format!("```rs\n{}\n```", report_errors(ctxt.raw_input, errs)))
                    .await?;
            },
        }

        Ok(())
    }
}
