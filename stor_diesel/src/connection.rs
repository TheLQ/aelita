use diesel::Connection;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use std::env;

pub fn load_db_url_from_env() -> String {
    dotenv().ok();
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

pub fn establish_connection() -> MysqlConnection {
    let database_url = load_db_url_from_env();
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
