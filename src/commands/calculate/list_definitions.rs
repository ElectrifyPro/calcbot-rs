use async_trait::async_trait;
use calcbot_attrs::Info;
use cas_compute::numerical::ctxt::{Func, UserFunc};
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lists all variables and functions defined using `{prefix}calculate`.
#[derive(Clone, Info)]
#[info(aliases = ["listdefs", "listdef", "ld", "ls"])]
pub struct ListDefinitions;

#[async_trait]
impl Command for ListDefinitions {
    async fn execute(
        &self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: &Context,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (vars, funcs) = {
            let mut database = database.lock().await;
            let user_data = database.get_user(ctxt.trigger.author_id()).await;

            (
                user_data.ctxt.get_vars()
                    .iter()
                    .map(|(name, value)| format!("`{} = {}`", name, value))
                    .collect::<Vec<_>>(),
                user_data.ctxt.get_funcs()
                    .values()
                    .filter_map(|func| match func {
                        Func::UserFunc(UserFunc { header, body, .. }) => Some(format!("`{} = {}`", header, body)),
                        Func::Builtin(_) => None,
                    })
                    .collect::<Vec<_>>(),
            )
        };

        ctxt.trigger.reply(&state.http)
            .content(&format!("**Variables**:\n{}\n\n**Functions**:\n{}", vars.join("\n"), funcs.join("\n")))?
            .await?;

        Ok(())
    }
}
