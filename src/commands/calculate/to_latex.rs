use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_parser::parser::{ast::expr::Expr, fmt::Latex, Parser};
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use strip_ansi_escapes::strip;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Converts an expression to LaTeX.
#[derive(Clone, Info)]
#[info(
    aliases = ["tolatex", "tolat", "latex", "tl"],
    syntax = ["<expression>"],
    examples = ["sin(root(2, 16) / 4)"],
    parent = super::Calculate,
)]
pub struct ToLatex;

#[async_trait]
impl Command for ToLatex {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let mut parser = Parser::new(ctxt.raw_input);
        match parser.try_parse_full::<Expr>() {
            Ok(expr) => {
                ctxt.trigger.reply(&state.http)
                    .content(&format!("**Converting** `{}` to LaTeX\n```{}```", ctxt.raw_input, expr.as_display()))
                    .await?;
            },
            Err(errs) => {
                let msg = errs.into_iter()
                    .map(|err| {
                        let mut buf = Vec::new();
                        err.build_report("input")
                            .write(("input", Source::from(ctxt.raw_input)), &mut buf)
                            .unwrap();
                        String::from_utf8(strip(buf).unwrap()).unwrap()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                ctxt.trigger.reply(&state.http)
                    .content(&format!("```{}```", msg))
                    .await?;
            },
        }

        Ok(())
    }
}
