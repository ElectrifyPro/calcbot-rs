use ariadne::Source;
use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_compiler::{Compiler, expr::compile_stmts};
use cas_parser::parser::{Parser, ast::LitSym};
use cas_vm::{Value, Vm};
use ordinal::ToOrdinal;
use crate::{
    arg_parse::{Remainder, parse_args_full},
    commands::{Command, Context, Info},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Calculates the summation of a sequence in terms of `n`, starting from the `a`-th term and
/// ending at the `b`-th term (inclusive).
#[derive(Clone, Info)]
#[info(
    aliases = ["sum", "s"],
    syntax = ["<a> <b> <expression>"],
    examples = ["1 24 5n - 3", "1 100 n", "3 8 2^n"],
    parent = super::Sequence,
)]
pub struct Sum;

#[async_trait]
impl Command for Sum {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        _database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let (a, b, expression) = parse_args_full::<(i64, i64, Remainder)>(ctxt.raw_input)
            .map_err(|err| if matches!(err, Error::NoArgument) {
                self.info().build_embed(ctxt.prefix).into()
            } else {
                err
            })?;

        if a >= b {
            return Err(Error::String("**The starting term `a` must be less than the ending term `b`.**".to_string()));
        }

        let expression = expression.0;

        let (n_id, mut vm) = {
            let stmts = match Parser::new(expression).try_parse_full_many() {
                Ok(stmts) => stmts,
                Err(errs) => Err(Error::CasMany(Source::from(expression), errs))?,
            };

            let mut compiler = Compiler::new();
            let n_id = compiler.add_symbol(&LitSym {
                name: "n".to_string(),
                span: 0..0,
            }).unwrap();
            match compile_stmts(&stmts, &mut compiler) {
                Ok(_) => {},
                Err(err) => Err(Error::Cas(Source::from(expression), err))?,
            }
            (n_id, Vm::from(compiler))
        };

        let source = Source::from(expression);

        let cancel = vm.stop_execution.clone();
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(4)) => {
                cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                ctxt.trigger.reply(&state.http)
                    .content("**Timeout: The calculation took too long (4+ seconds) and was cancelled.** Check your expression to see if there are any mistakes or infinite loops.")
                    .await?;
            }
            Ok(result) = tokio::task::spawn_blocking(move || {
                let mut sum = 0.0;
                let mut a_value = 0.0;
                let mut b_value = 0.0;

                for n in a..=b {
                    vm.variables.insert(n_id, n.into());
                    let value = match vm.run() {
                        Ok(val) => match val.coerce_number() {
                            Value::Integer(i) => i.to_f64(),
                            Value::Float(f) => f.to_f64(),
                            _ => return Err(Error::String("**The expression did not evaluate to a number.** Please ensure that the expression results in a numeric value for all values of `n` in the specified range.".to_string())),
                        },
                        Err(err) => return Err(Error::Cas(source, err)),
                    };

                    sum += value;
                    if n == a {
                        a_value = value;
                    }
                    if n == b {
                        b_value = value;
                    }
                }

                Ok((sum, a_value, b_value))
            }) => {
                let (sum, a_value, b_value) = result?;
                let a = a.to_ordinal();
                let b = b.to_ordinal();
                ctxt.trigger.reply(&state.http)
                    .content(&format!("**Summation** of **{a}** to **{b}** terms ({a_value} + ... + {b_value}) of `{expression}`\n{sum}"))
                    .await?;
            }
        }

        Ok(())
    }
}
