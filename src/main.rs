use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{config::get_config, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_config().expect("Failed to get configurations");

    let db_connection_string = config.database.connection_string();
    let db_pool = PgPool::connect(&db_connection_string)
        .await
        .expect("Failed to connect to database.");

    let address = format! {"127.0.0.1:{}", config.application_port};
    let listener = TcpListener::bind(address).expect("Failed to bind address.");

    run(listener, db_pool)?.await
}
