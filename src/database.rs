use dotenv::var;
use mysql_async::{
    prelude::{Query, WithParams},
    OptsBuilder,
    Pool,
};
use std::collections::HashMap;
use twilight_model::id::{Id, marker::GuildMarker};

/// Helper struct to access and manage the database.
pub struct Database {
    /// A connection pool to the database.
    pool: Pool,

    /// The server cache. This stores the prefix of CalcBot on servers that have recently used it.
    server_cache: HashMap<Id<GuildMarker>, String>,
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
            server_cache: HashMap::new(),
        }
    }

    /// Returns the data of the server with the given ID.
    ///
    /// If the data was cached previously, the cached value will be returned. Otherwise, the data
    /// will be fetched from the database, cached, then returned.
    ///
    /// If the data does not exist anywhere, a default is created.
    pub async fn get_server(&mut self, id: Id<GuildMarker>) -> &str {
        if self.server_cache.contains_key(&id) {
            return &self.server_cache[&id];
        }

        let prefix = match "SELECT id, prefix FROM servers WHERE id = ? LIMIT 1"
            .with((id.get(),))
            .first::<(u64, String), _>(&self.pool)
            .await
            .unwrap()
        {
            Some((_, prefix)) => prefix,
            None => {
                "INSERT INTO servers (id, prefix) VALUES (?, 'c-')"
                    .with((id.get(),))
                    .ignore(&self.pool)
                    .await
                    .unwrap();
                String::from("c-")
            },
        };

        self.server_cache.entry(id).or_insert(prefix)
    }
}
