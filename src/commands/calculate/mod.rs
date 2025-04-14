pub mod list_definitions;
pub mod mode;
pub mod to_latex;

use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_compute::numerical::eval::eval_stmts;
use cas_parser::parser::Parser;
use crate::{
    commands::{Command, Context},
    database::{user::Ctxt, Database},
    error::Error,
    global::State,
};
use strip_ansi_escapes::strip;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Evaluates a given expression, like `1 + 1`. You can declare variables by typing `variablename =
/// [value]`.
///
/// You can find extended documentation for this command
/// [here](https://chillant.gitbook.io/calcbot/commands/calculate).
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
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let mut parser = Parser::new(ctxt.raw_input);
        match parser.try_parse_full_many() {
            Ok(stmts) => {
                let mut database = database.lock().await;
                let eval_ctxt = database.get_user_field_mut::<Ctxt>(ctxt.trigger.author_id()).await;

                let ans = match eval_stmts(&stmts, eval_ctxt) {
                    Ok(ans) => ans,
                    Err(err) => {
                        let mut buf = Vec::new();
                        err.build_report()
                            .write(("input", Source::from(ctxt.raw_input)), &mut buf)
                            .unwrap();

                        ctxt.trigger.reply(&state.http)
                            .content(&format!("```rs\n{}\n```", String::from_utf8_lossy(&strip(buf).unwrap())))?
                            .await?;
                        return Ok(());
                    },
                };
                ctxt.trigger.reply(&state.http)
                    .content(&format!("**Calculation** (mode: {})\n{}", eval_ctxt.trig_mode, ans))?
                    .await?;

                eval_ctxt.add_var("ans", ans);
                database.commit_user_field::<Ctxt>(ctxt.trigger.author_id()).await;
            },
            Err(errs) => {
                let msg = errs.into_iter()
                    .map(|err| {
                        let mut buf = Vec::new();
                        err.build_report()
                            .write(("input", Source::from(ctxt.raw_input)), &mut buf)
                            .unwrap();
                        String::from_utf8(strip(buf).unwrap()).unwrap()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                ctxt.trigger.reply(&state.http)
                    .content(&format!("```rs\n{}\n```", msg))?
                    .await?;
            },
        }

        Ok(())
    }
}
