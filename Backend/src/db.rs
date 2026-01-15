use mongodb::{Client, Database};


#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub fn new(client: Client, db_name: &str) -> Self {
        let db = client.database(db_name);
        Self { db }
    }
}

pub async fn mongo_client(uri: &str) -> Client {
    Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB")
}
