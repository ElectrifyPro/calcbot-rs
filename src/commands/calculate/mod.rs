pub mod list_definitions;
pub mod mode;
pub mod to_latex;

use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_eval::eval::Eval;
use cas_parser::parser::{ast::expr::Expr, Parser};
use crate::{
    commands::{Command, Context},
    database::{user::UserField, Database},
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
    async fn execute(
        &self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut parser = Parser::new(ctxt.raw_input);
        match parser.try_parse_full::<Expr>() {
            Ok(expr) => {
                let mut user_data = database.lock().await
                    .get_user(ctxt.message.author.id).await
                    .clone();

                let ans = match expr.eval(&mut user_data.ctxt) {
                    Ok(ans) => ans,
                    Err(err) => {
                        let mut buf = Vec::new();
                        err.build_report()
                            .write(("input", Source::from(ctxt.raw_input)), &mut buf)
                            .unwrap();

                        state.http.create_message(ctxt.message.channel_id)
                            .content(&format!("```{}```", String::from_utf8_lossy(&strip(buf).unwrap())))?
                            .await?;
                        return Ok(());
                    },
                };
                state.http.create_message(ctxt.message.channel_id)
                    .content(&format!("**Calculation** (mode: {})\n{}", user_data.ctxt.trig_mode, ans))?
                    .await?;

                user_data.ctxt.add_var("ans", ans);
                database.lock().await
                    .set_user_field(ctxt.message.author.id, UserField::Ctxt(user_data.ctxt)).await;
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

                state.http.create_message(ctxt.message.channel_id)
                    .content(&format!("```{}```", msg))?
                    .await?;
            },
        }

        Ok(())
    }
}
