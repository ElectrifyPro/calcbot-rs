use crate::{database::{Database, user::Timers}, global::State, timer::Timer};
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

/// Reason for completing a timer.
pub enum Reason {
    Triggered,
    Deleted,
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Triggered => write!(f, "(completed)"),
            Reason::Deleted => write!(f, "(deleted)"),
        }
    }
}

/// Handles the actions taken when a timer is finished (triggered, deleted, etc.). This includes:
///
/// - Removing it from the database.
/// - Removing the confirmation message mapping if it was a shared reminder.
/// - Editing the confirmation message button to be disabled.
pub async fn complete(
    timer_id: &str,
    user_id: Id<UserMarker>,
    state: &State,
    db: &mut Database,
    reason: Reason,
) -> Option<Timer> {
    let timer = db.get_user_field_mut::<Timers>(user_id).await.remove(timer_id)?;
    db.commit_user_field::<Timers>(user_id).await;

    let Some(confirmation_message_id) = timer.confirmation_message_id else {
        return Some(timer);
    };

    db.remove_shared_reminder(confirmation_message_id).await;

    let num_subscribers = timer.subscribed_users.len();
    let button_label = if num_subscribers == 0 {
        "Remind me".to_string()
    } else {
        format!(
            "Remind me ({num_subscribers} user{})",
            if num_subscribers == 1 { "" } else { "s" },
        )
    };

    let _ = state.http.update_message(timer.channel_id, confirmation_message_id)
        .components(Some(&[
            Component::ActionRow(ActionRow {
                components: vec![
                    Component::Button(Button {
                        custom_id: Some("remind-me-too".to_owned()),
                        disabled: true,
                        emoji: Some(EmojiReactionType::Unicode {
                            name: String::from("⏰"),
                        }),
                        label: Some(format!("{button_label} {reason}")),
                        style: ButtonStyle::Primary,
                        url: None,
                        sku_id: None,
                    }),
                ],
            }),
        ]))
        .await;

    Some(timer)
}

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
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(InteractionResponseDataBuilder::new()
                        .content("**You are the author of this reminder and will always be pinged when it triggers.**")
                        .flags(MessageFlags::EPHEMERAL)
                        .build()),
                },
            )
        .await?;
        return Ok(());
    }

    let Some(timer) = db_lock.get_user_field_mut::<Timers>(reminder_author).await
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
