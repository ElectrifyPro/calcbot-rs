use async_trait::async_trait;
use calcbot_attrs::Info;
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
            let user_data = database.get_user(ctxt.message.author.id).await;

            (
                user_data.ctxt.get_vars()
                    .iter()
                    .map(|(name, value)| format!("`{} = {}`", name, value))
                    .collect::<Vec<_>>(),
                user_data.ctxt.get_funcs()
                    .values()
                    .map(|func| format!("`{} = {}`", func.header, func.body))
                    .collect::<Vec<_>>(),
            )
        };

        state.http.create_message(ctxt.message.channel_id)
            .content(&format!("**Variables**:\n{}\n\n**Functions**:\n{}", vars.join("\n"), funcs.join("\n")))?
            .await?;

        Ok(())
    }
}
