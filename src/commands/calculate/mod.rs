use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_eval::eval::Eval;
use cas_parser::parser::{expr::Expr, Parser};
use crate::{
    commands::{Command, Context},
    database::Database,
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
    examples = ["1+1", "x=2", "5sin(pi/2)", "6!", "f(x)=x^2+5x+6", "f(2)", "cos'(0)"]
)]
pub struct Calculate;

#[async_trait]
impl Command for Calculate {
    async fn execute(
        &self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut parser = Parser::new(ctxt.raw_input);
        match parser.try_parse_full::<Expr>() {
            Ok(expr) => {
                let ans = expr.eval_default().unwrap();
                state.http.create_message(ctxt.message.channel_id)
                    .content(&format!("**Calculation** (mode: degrees)\n{}", ans))?
                    .await?;
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
