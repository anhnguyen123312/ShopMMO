//! MongoDB database connection and management
//!
//! Handles MongoDB client initialization, connection pooling, and provides
//! database access throughout the application.

use mongodb::{
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Database,
};
use std::time::Duration;

use crate::config::AppConfig;

/// MongoDB database wrapper
///
/// Provides access to MongoDB collections and handles connection management.
#[derive(Clone)]
pub struct MongoDB {
    client: Client,
    database: Database,
}

impl MongoDB {
    /// Creates a new MongoDB connection
    ///
    /// # Arguments
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `Result<Self, mongodb::error::Error>` - MongoDB instance or error
    ///
    /// # Examples
    /// ```
    /// let config = AppConfig::from_env()?;
    /// let db = MongoDB::connect(&config).await?;
    /// ```
    pub async fn connect(config: &AppConfig) -> Result<Self, mongodb::error::Error> {
        tracing::info!(
            uri = %config.database.mongodb_uri,
            database = %config.database.mongodb_database,
            "Connecting to MongoDB"
        );

        // Parse connection string
        let mut client_options = ClientOptions::parse(&config.database.mongodb_uri).await?;

        // Set server API version
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        // Set connection pool options
        client_options.max_pool_size = Some(config.database.mongodb_max_pool_size);
        client_options.min_pool_size = Some(config.database.mongodb_min_pool_size);

        // Set timeout options
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.server_selection_timeout = Some(Duration::from_secs(10));

        // Set application name for connection tracking
        client_options.app_name = Some("mmo-api".to_string());

        // Create client
        let client = Client::with_options(client_options)?;

        // Verify connection with ping
        client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 })
            .await?;

        tracing::info!("Successfully connected to MongoDB");

        let database = client.database(&config.database.mongodb_database);

        Ok(Self { client, database })
    }

    /// Gets the MongoDB database instance
    ///
    /// # Returns
    /// * `&Database` - Reference to the database
    ///
    /// # Examples
    /// ```
    /// let db = mongodb.database();
    /// let collection = db.collection::<User>("users");
    /// ```
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Gets the MongoDB client instance
    ///
    /// # Returns
    /// * `&Client` - Reference to the client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Creates a new client session for transactions
    ///
    /// # Returns
    /// * `Result<ClientSession, mongodb::error::Error>` - Session or error
    ///
    /// # Examples
    /// ```
    /// let session = db.start_session().await?;
    /// session.start_transaction(None).await?;
    /// // ... perform operations with session
    /// session.commit_transaction().await?;
    /// ```
    pub async fn start_session(
        &self,
    ) -> Result<mongodb::ClientSession, mongodb::error::Error> {
        self.client.start_session().await
    }

    /// Checks database health
    ///
    /// # Returns
    /// * `Result<bool, mongodb::error::Error>` - True if healthy
    ///
    /// # Examples
    /// ```
    /// if db.health_check().await? {
    ///     println!("Database is healthy");
    /// }
    /// ```
    pub async fn health_check(&self) -> Result<bool, mongodb::error::Error> {
        self.client
            .database("admin")
            .run_command(mongodb::bson::doc! { "ping": 1 })
            .await?;
        Ok(true)
    }
}

/// MongoDB collection names as constants
///
/// Centralized collection name management to avoid typos and maintain consistency.
pub mod collections {
    /// Users collection
    pub const USERS: &str = "users";

    /// Refresh tokens collection
    pub const REFRESH_TOKENS: &str = "refresh_tokens";

    /// Wallets collection
    pub const WALLETS: &str = "wallets";

    /// Wallet transactions collection
    pub const WALLET_TRANSACTIONS: &str = "wallet_transactions";

    /// Escrow holds collection
    pub const ESCROW_HOLDS: &str = "escrow_holds";

    /// Withdrawal requests collection
    pub const WITHDRAWAL_REQUESTS: &str = "withdrawal_requests";

    /// Deposit requests collection
    pub const DEPOSIT_REQUESTS: &str = "deposit_requests";

    /// Order type configs collection
    pub const ORDER_TYPE_CONFIGS: &str = "order_type_configs";

    /// Money flow summary collection
    pub const MONEY_FLOW_SUMMARY: &str = "money_flow_summary";

    /// Orders collection
    pub const ORDERS: &str = "orders";
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests should be run with a test database
    // Example: MONGODB_URI=mongodb://localhost:27017/mmo_test cargo test
}
