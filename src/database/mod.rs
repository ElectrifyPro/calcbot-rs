mod const_str;
pub mod user;

use crate::global::State;
use const_str::ConstStr;
use dotenv::var;
use mysql_async::{
    prelude::{Query, WithParams},
    Error,
    OptsBuilder,
    Pool,
};
use serde_json::to_value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, Mutex};
use twilight_model::{
    gateway::payload::incoming::InteractionCreate,
    id::{Id, marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker}},
};
use user::{UserData, UserField, UserSettings};

/// Helper struct to access and manage the database.
pub struct Database {
    /// A connection pool to the database.
    pool: Pool,

    /// The server cache. This stores the prefix of CalcBot on servers that have recently used it.
    servers: HashMap<Id<GuildMarker>, String>,

    /// The user cache. This stores the user data of users that have recently used CalcBot.
    users: HashMap<Id<UserMarker>, UserData>,

    /// The user settings cache. This stores the settings of users that have recently used CalcBot.
    user_settings: HashMap<Id<UserMarker>, UserSettings>,

    /// Paged messages that are currently being displayed.
    paged: HashMap<(Id<ChannelMarker>, Id<MessageMarker>), UnboundedSender<InteractionCreate>>,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            pool: Pool::new(
                OptsBuilder::default()
                    .user(var("MYSQL_USER").ok())
                    .ip_or_hostname(var("MYSQL_HOST").unwrap())
                    .pass(var("MYSQL_PASS").ok()) // in our case, password not needed during dev, but required for prod
                    .tcp_port(3306)
                    .db_name(Some("calcbot"))
                    .socket(var("MYSQL_SOCKET").ok())
            ),
            servers: HashMap::new(),
            users: HashMap::new(),
            user_settings: HashMap::new(),
            paged: HashMap::new(),
        }
    }

    /// Sets the paged message sender for the given channel and message IDs. This is used to listen
    /// for interactions on messages with multiple pages.
    pub fn set_paged_message(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> UnboundedReceiver<InteractionCreate> {
        let (sender, receiver) = unbounded_channel();
        self.paged.insert((channel_id, message_id), sender);
        receiver
    }

    /// Obtains the paged message sender for the given channel and message IDs.
    ///
    /// If the sender is closed (the receiver has been dropped), the sender will automatically be
    /// removed from the cache and [`None`] will be returned.
    pub fn get_paged_message(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> Option<&UnboundedSender<InteractionCreate>> {
        let sender_is_closed = self.paged
            .get(&(channel_id, message_id))
            .is_some_and(|sender| sender.is_closed());
        if sender_is_closed {
            self.paged.remove(&(channel_id, message_id));
            None
        } else {
            self.paged.get(&(channel_id, message_id))
        }
    }

    /// Removes the paged message sender for the given channel and message IDs. Returns `true` if
    /// the sender was removed.
    pub fn remove_paged_message(
        &mut self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> bool {
        self.paged.remove(&(channel_id, message_id)).is_some()
    }

    /// Returns the data of the server with the given ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    pub async fn get_server(&mut self, id: Id<GuildMarker>) -> Result<&str, Error> {
        if self.servers.contains_key(&id) {
            return Ok(&self.servers[&id]);
        }

        let prefix = match "SELECT prefix FROM servers WHERE id = ? LIMIT 1"
            .with((id.get(),))
            .first::<String, _>(&self.pool)
            .await?
        {
            Some(prefix) => prefix,
            None => {
                "INSERT INTO servers (id, prefix) VALUES (?, 'c-')"
                    .with((id.get(),))
                    .ignore(&self.pool)
                    .await?;
                String::from("c-")
            },
        };

        Ok(self.servers.entry(id).or_insert(prefix))
    }

    /// Gets immutable access to the user settings for the given user ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    pub async fn get_user_settings(&mut self, id: Id<UserMarker>) -> &UserSettings {
        if self.user_settings.contains_key(&id) {
            return self.user_settings.get(&id).unwrap();
        }

        let settings = match "SELECT time_zone FROM user_settings WHERE id = ? LIMIT 1"
            .with((id.get(),))
            .first::<UserSettings, _>(&self.pool)
            .await
            .unwrap()
        {
            Some(settings) => settings,
            None => {
                "INSERT INTO user_settings (id, time_zone) VALUES (?, ?)"
                    .with((id.get(), i8::default()))
                    .ignore(&self.pool)
                    .await
                    .unwrap();
                UserSettings::default()
            },
        };

        self.user_settings.entry(id).or_insert(settings)
    }

    /// Loads all users that have timers set in the database, allowing timers to continue running
    /// even if the bot is restarted.
    pub async fn resume_users_with_timers(&mut self, state: Arc<State>, db: Arc<Mutex<Database>>) {
        let users = "SELECT id, ctxt, timers FROM users WHERE timers != '{}'"
            .fetch::<UserData, _>(&self.pool)
            .await
            .unwrap();

        for mut user in users {
            for timer in user.timers.values_mut() {
                timer.create_task(Arc::clone(&state), Arc::clone(&db));
            }
            self.users.insert(user.id, user);
        }
    }
    /// Gets mutable access to the user data for the given user ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    async fn get_user_mut(&mut self, id: Id<UserMarker>) -> &mut UserData {
        if self.users.contains_key(&id) {
            return self.users.get_mut(&id).unwrap();
        }

        let data = match "SELECT id, ctxt, timers FROM users WHERE id = ? LIMIT 1"
            .with((id.get(),))
            .first::<UserData, _>(&self.pool)
            .await
            .unwrap()
        {
            Some(data) => data,
            None => {
                "INSERT INTO users (id, ctxt, timers) VALUES (?, ?, ?)"
                    .with((
                        id.get(),
                        to_value(cas_compute::numerical::ctxt::Ctxt::default()).unwrap(),
                        to_value(HashMap::<(), ()>::new()).unwrap(),
                    ))
                    .ignore(&self.pool)
                    .await
                    .unwrap();
                UserData::new(id)
            },
        };

        self.users.entry(id).or_insert(data)
    }

    /// Gets immutable access to the user data for the given user ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    pub async fn get_user(&mut self, id: Id<UserMarker>) -> &UserData {
        self.get_user_mut(id).await
    }

    /// Gets mutable access to the specified field of the user data for the given user ID.
    ///
    /// After modifying the data, use [`Database::commit_user_field`] to commit the changes to the
    /// database.
    pub async fn get_user_field_mut<'a, T: UserField>(
        &'a mut self,
        id: Id<UserMarker>,
    ) -> &'a mut T::Type
    where
        T::Type: 'a,
    {
        let user = self.get_user_mut(id).await;
        T::get_mut(user)
    }

    /// Commits changes made to the specified field of the user data for the given user ID to the
    /// database.
    ///
    /// After calling [`Database::get_user_field_mut`], call this function to commit the changes to
    /// the database.
    pub async fn commit_user_field<T: UserField>(&mut self, id: Id<UserMarker>) {
        let user = self.get_user_mut(id).await;
        const { ConstStr::new()
            .append("UPDATE users SET ")
            .append(T::COLUMN_NAME)
            .append(" = ? WHERE id = ?") }
            .as_str()
            .with((
                to_value(T::get_mut(user)).unwrap(),
                id.get(),
            ))
            .ignore(&self.pool)
            .await
            .unwrap();
    }
}
