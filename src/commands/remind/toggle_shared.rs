use crate::{database::{Database, user::Timers}, global::State};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::{
    channel::message::{
        Component,
        EmojiReactionType,
        Message,
        MessageFlags,
        component::{ActionRow, Button, ButtonStyle},
    },
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{Id, marker::UserMarker},
};
use twilight_util::builder::InteractionResponseDataBuilder;

/// Register or unregister the interacting user to receive a copy of a reminder
/// set by another user.
pub async fn toggle_shared(
    interaction: &InteractionCreate,
    state: &Arc<State>,
    database: &Arc<Mutex<Database>>,
    db_lock: &mut Database,
    reminder_author: Id<UserMarker>,
    timer_id: &str,
    original_confirmation_message: &Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let Some(interacting_user_id) = interaction.author_id() else {
        return Ok(());
    };

    if reminder_author == interacting_user_id {
        state.http.interaction(state.application_id)
            .create_response(
                interaction.id,
                &interaction.token,
                &twilight_model::http::interaction::InteractionResponse {
                    kind: twilight_model::http::interaction::InteractionResponseType::ChannelMessageWithSource,
                    data: Some(twilight_util::builder::InteractionResponseDataBuilder::new()
                        .content("**You are the author of this reminder and will always be pinged when it triggers.**")
                        .flags(MessageFlags::EPHEMERAL)
                        .build()),
                },
            )
        .await?;
        return Ok(());
    }

    let Some(timer) = dbg!(db_lock.get_user_field_mut::<Timers>(reminder_author).await)
        .get_mut(timer_id) else {
        return Ok(());
    };
    timer.toggle_subscribed_user(interacting_user_id);
    timer.create_task(Arc::clone(state), Arc::clone(database));

    let num_subscribers = timer.subscribed_users.len();
    let (button_label, notification) = if num_subscribers == 0 {
        (
            "Remind me".to_string(),
            "**Success:** You will _no longer_ receive this reminder.",
        )
    } else {
        (
            format!(
                "Remind me ({num_subscribers} user{})",
                if num_subscribers == 1 { "" } else { "s" },
            ),
            "**Success:** You _will_ also be pinged when this reminder reminder triggers.",
        )
    };

    state.http.interaction(state.application_id)
        .create_response(
            interaction.id,
            &interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::UpdateMessage,
                data: Some(InteractionResponseDataBuilder::new()
                    .content(original_confirmation_message.content.clone())
                    .components(vec![
                        Component::ActionRow(ActionRow {
                            components: vec![
                                Component::Button(Button {
                                    custom_id: Some("remind-me-too".to_owned()),
                                    disabled: false,
                                    emoji: Some(EmojiReactionType::Unicode {
                                        name: String::from("⏰"),
                                    }),
                                    label: Some(button_label),
                                    style: ButtonStyle::Primary,
                                    url: None,
                                    sku_id: None,
                                }),
                            ],
                        }),
                    ])
                    .build()),
            },
        )
        .await?;
    state.http.interaction(state.application_id)
        .create_followup(&interaction.token)
        .content(notification)
        .flags(MessageFlags::EPHEMERAL)
        .await?;
    Ok(())
}
