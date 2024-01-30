use std::net::TcpListener;

use news_letter::configuration::get_configuration;
use sqlx::PgPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres.");

    let listener = TcpListener::bind(address)?;
    news_letter::startup::run(listener, connection_pool)?.await
}
