pub mod user;

use crate::timer::Timer;
use dotenv::var;
use mysql_async::{
    prelude::{Query, WithParams},
    Error,
    OptsBuilder,
    Pool,
};
use serde_json::to_value;
use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use twilight_model::{
    gateway::payload::incoming::InteractionCreate,
    id::{Id, marker::{ChannelMarker, GuildMarker, MessageMarker, UserMarker}},
};
use user::{UserData, UserField};

/// Helper struct to access and manage the database.
pub struct Database {
    /// A connection pool to the database.
    pool: Pool,

    /// The server cache. This stores the prefix of CalcBot on servers that have recently used it.
    servers: HashMap<Id<GuildMarker>, String>,

    /// The user cache. This stores the user data of users that have recently used CalcBot.
    users: HashMap<Id<UserMarker>, UserData>,

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
            .map_or(false, |sender| sender.is_closed());
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

    /// Returns the user data for the given user ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    pub async fn get_user(&mut self, id: Id<UserMarker>) -> &UserData {
        if self.users.contains_key(&id) {
            return &self.users[&id];
        }

        let data = match "SELECT ctxt, timers FROM users WHERE id = ? LIMIT 1"
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
                UserData::default()
            },
        };

        self.users.entry(id).or_insert(data)
    }

    /// Sets the user data for the given user ID.
    ///
    /// This will update the cached value and the database value.
    pub async fn set_user(&mut self, id: Id<UserMarker>, data: UserData) {
        "UPDATE users SET ctxt = ?, timers = ? WHERE id = ?"
            .with((
                to_value(&data.ctxt).unwrap(),
                to_value(&data.timers).unwrap(),
                id.get(),
            ))
            .ignore(&self.pool)
            .await
            .unwrap();
        self.users.insert(id, data);
    }

    /// Sets a specific field of the user data for the given user ID.
    ///
    /// This will update the cached value and the database value.
    pub async fn set_user_field(&mut self, id: Id<UserMarker>, field: UserField) {
        match field {
            UserField::Ctxt(ctxt) => {
                "UPDATE users SET ctxt = ? WHERE id = ?"
                    .with((to_value(&ctxt).unwrap(), id.get()))
                    .ignore(&self.pool)
                    .await
                    .unwrap();
                self.users.get_mut(&id).unwrap().ctxt = ctxt;
            },
            UserField::Timers(timers) => {
                "UPDATE users SET timers = ? WHERE id = ?"
                    .with((to_value(&timers).unwrap(), id.get()))
                    .ignore(&self.pool)
                    .await
                    .unwrap();
                self.users.get_mut(&id).unwrap().timers = timers;
            },
        }
    }

    /// Add a managed timer to the database.
    pub async fn add_timer(&mut self, timer: Timer) {
        let user_id = timer.user_id.clone();
        let mut user_timers = self.get_user(timer.user_id)
            .await
            .timers
            .clone();
        user_timers.insert(timer.id.clone(), timer);

        self.set_user_field(user_id, UserField::Timers(user_timers)).await;
    }

    /// Remove a managed timer from the database. Returns the removed instance.
    pub async fn remove_timer(&mut self, id: &Id<UserMarker>, timer_id: &str) -> Option<Timer> {
        let user = self.users.get_mut(id)?;
        user.timers.remove(timer_id)
    }

    /// Increments a managed timer in the database.
    pub async fn increment_timer(
        &mut self,
        id: &Id<UserMarker>,
        timer_id: &str,
        duration: Duration,
    ) -> Option<&Timer> {
        let user = self.users.get_mut(id)?;
        let timer = user.timers.get_mut(timer_id)?;
        *timer += duration;

        Some(timer)
    }
}
