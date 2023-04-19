use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::Command,
    database::Database,
    global::State,
};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::channel::message::Message;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

/// Access various useful links for CalcBot, such as online documentation or CalcBot's invite link.
#[derive(Clone, Info)]
#[info(category = "Resources")]
pub struct Link;

#[async_trait]
impl Command for Link {
    async fn execute(
        &self,
        state: Arc<State>,
        _: Arc<Mutex<Database>>,
        message: &Message,
        _: Vec<&str>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let embed = EmbedBuilder::new()
            .title("Links")
            .color(0x641964)
            .description("You can join my support server [here](https://discord.com/invite/3m7dK92).")
            .field(EmbedFieldBuilder::new(
                "top.gg",
                "[View](https://top.gg/bot/674457690646249472)\n[Vote](https://top.gg/bot/674457690646249472/vote)",
            ).inline())
            .field(EmbedFieldBuilder::new(
                "BOD",
                "[View](https://bots.ondiscord.xyz/bots/674457690646249472)\n[Review](https://bots.ondiscord.xyz/bots/674457690646249472/review)",
            ).inline())
            .field(EmbedFieldBuilder::new(
                "BFD",
                "[View](https://botsfordiscord.com/bot/674457690646249472)\n[Vote](https://botsfordiscord.com/bot/674457690646249472/vote)",
            ).inline())
            .field(EmbedFieldBuilder::new(
                "Online documentation",
                "[GitBook](https://chillant.gitbook.io/calcbot/)",
            ).inline())
            .field(EmbedFieldBuilder::new(
                "Invite me",
                "[Here](https://discordapp.com/api/oauth2/authorize?client_id=674457690646249472&permissions=109568&scope=bot)",
            ).inline())
            .field(EmbedFieldBuilder::new(
                "Submit a private request",
                "[Google form](https://forms.gle/uiWsWgseGLhZtWow9)",
            ).inline())
            .build();

        state.http.create_message(message.channel_id)
            .embeds(&[embed])
            .unwrap()
            .await?;

        Ok(())
    }
}