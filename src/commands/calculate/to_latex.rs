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
            Err(errs) => Err(Error::CasMany(Source::from(ctxt.raw_input), errs))?,
        }

        Ok(())
    }
}
