use async_trait::async_trait;
use calcbot_attrs::Info;
use crate::{
    commands::{Command, Context},
    database::Database,
    error::Error,
    global::State,
    util::Clamped,
};
use serde::{Deserialize, Serialize};
use std::{future::IntoFuture, sync::Arc};
use tokio::sync::Mutex;
use twilight_model::{
    application::interaction::InteractionData,
    channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
        Embed,
        EmojiReactionType,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ChannelMarker, Id},
};
use twilight_util::builder::{
    embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder},
    InteractionResponseDataBuilder,
};

lazy_static::lazy_static! {
    /// List of all supported units.
    static ref UNITS: Vec<Quantity> = {
        let units = include_str!("./units.json");
        serde_json::from_str(units).unwrap()
    };
}

/// A quantity kind, like length or time.
#[derive(Deserialize, Serialize)]
struct Quantity {
    /// The name of the quantity.
    kind: String,

    /// The units of the quantity.
    units: Vec<Unit>,
}

/// A unit of a quantity.
#[derive(Deserialize, Serialize)]
struct Unit {
    /// The abbreviation of the unit.
    abbreviation: String,

    /// The full name of the unit.
    name: String,
}

/// Sends a Discord message that has multiple pages split as embeds. A task is spawned to listen
/// for button clicks and update the message accordingly.
fn send_paged_message(
    state: &Arc<State>,
    database: &Arc<Mutex<Database>>,
    channel_id: Id<ChannelMarker>,
    pages: &[Embed],
    index: usize,
) -> Result<(), Error> {
    // validate before sending
    let component = Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                custom_id: Some("prev".to_owned()),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode {
                    name: String::from("◀️"),
                }),
                label: Some(String::from("Previous")),
                style: ButtonStyle::Primary,
                url: None,
                sku_id: None,
            }),
            Component::Button(Button {
                custom_id: Some("next".to_owned()),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode {
                    name: String::from("▶️"),
                }),
                label: Some(String::from("Next")),
                style: ButtonStyle::Primary,
                url: None,
                sku_id: None,
            }),
            Component::Button(Button {
                custom_id: Some("delete".to_owned()),
                disabled: false,
                emoji: Some(EmojiReactionType::Unicode {
                    name: String::from("🗑️"),
                }),
                label: Some(String::from("Delete")),
                style: ButtonStyle::Danger,
                url: None,
                sku_id: None,
            }),
        ],
    });
    let pages = pages.to_vec();
    let msg = state.http.create_message(channel_id)
        .embeds(&[pages[index].clone()])
        .components(&[component.clone()])
        .into_future();

    let state = Arc::clone(state);
    let database = Arc::clone(database);
    tokio::task::spawn(async move {
        let mut clamped = Clamped::new(index, pages.len());
        let message = msg.await?.model().await?;
        let mut receiver = database.lock().await.set_paged_message(channel_id, message.id);

        // TODO: if the message is manually deleted (not through the delete button), the receiver
        // and sender will not be dropped, which can cause wasted memory
        //
        // we need to listen for that message delete event
        while let Some(mut interaction) = receiver.recv().await {
            if let Some(InteractionData::MessageComponent(component_interaction)) = interaction.data.take() {
                match component_interaction.custom_id.as_str() {
                    "prev" => clamped -= 1,
                    "next" => clamped += 1,
                    "delete" => {
                        state.http.delete_message(channel_id, message.id).await?;
                        break;
                    },
                    _ => unreachable!(),
                }
                let new_embed = pages[*clamped].clone();
                state.http.interaction(state.application_id)
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::UpdateMessage,
                            data: Some(InteractionResponseDataBuilder::new()
                                .components(Some(component.clone()))
                                .embeds(vec![new_embed])
                                .build()),
                        },
                    )
                    .await?;
            }
        }

        log::info!("paged message task ended: delete interaction button clicked");

        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });

    Ok(())
}

/// Creates a embed builder with the common fields set.
fn create_embed(index: usize, total_pages: usize) -> EmbedBuilder {
    EmbedBuilder::new()
        .title("Supported units (case sensitive)")
        .color(0xed9632)
        .footer(EmbedFooterBuilder::new(format!("Page {} of {}", index + 1, total_pages)))
}

/// Generates embeds for the supported units.
fn generate_embeds() -> Vec<Embed> {
    let mut vec = Vec::new();

    for (i, quantity) in UNITS.iter().enumerate() {
        let begin = match quantity.kind.as_str() {
            "Area" => Some("Use a length unit^2 to represent area, or:".to_owned()),
            _ => None,
        };
        let abbreviations = begin
            .into_iter()
            .chain(
                quantity.units.iter()
                    .map(|unit| format!("`{}` - {}", unit.abbreviation, unit.name))
            )
            .collect::<Vec<_>>()
            .join("\n");
        let embed = create_embed(i, UNITS.len())
            .field(EmbedFieldBuilder::new(&quantity.kind, abbreviations).inline());
        vec.push(embed.build());
    }

    vec
}

/// Show a list of units supported by the unit conversion command.
#[derive(Clone, Info)]
#[info(
    aliases = ["units", "unit", "u"],
    syntax = ["[page number]"],
    parent = super::UnitConvert,
)]
pub struct Units;

#[async_trait]
impl Command for Units {
    async fn execute<'c>(
        &'c self,
        state: &Arc<State>,
        database: &Arc<Mutex<Database>>,
        ctxt: Context<'c>,
    ) -> Result<(), Error> {
        let index = ctxt.raw_input.parse::<usize>().unwrap_or(1).saturating_sub(1);
        let embeds = generate_embeds();
        send_paged_message(state, database, ctxt.trigger.channel_id(), &embeds, index)?;
        Ok(())
    }
}
