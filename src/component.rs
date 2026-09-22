use twilight_model::{channel::message::Component, id::{Id, marker::UserMarker}};

/// Returns an ActionRow component, containing one Delete button that the specified user can click
/// to delete the message it is attached to.
pub fn delete_row(user: Id<UserMarker>) -> Component {
    use twilight_model::channel::message::{
        component::{ActionRow, Button, ButtonStyle},
        Component,
        EmojiReactionType,
    };
    Component::ActionRow(ActionRow {
        components: vec![
            Component::Button(Button {
                custom_id: Some(format!("delete:{user}")),
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
    })
}
