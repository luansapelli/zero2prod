use actix_web::http::StatusCode;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::{config::get_config, startup::run};

#[tokio::test]
async fn health_check_api_should_return_status_ok() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}

#[tokio::test]
async fn subscription_api_should_return_status_ok_for_valid_form_data() {
    // Arrange
    let address = spawn_app();
    let config = get_config().expect("Failed to get configurations");
    let db_connection_string = config.database.connection_string();
    let mut db_connection = PgConnection::connect(&db_connection_string)
        .await
        .expect("Failed to connect to database.");
    let client = reqwest::Client::new();

    // Act
    let body = "name=jhon%20doe&email=jhondoe%40email.com";
    let response = client
        .post(&format!("{}/subscribe", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(StatusCode::OK, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut db_connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "jhondoe@email.com");
    assert_eq!(saved.name, "jhon doe");
}

#[tokio::test]
async fn subscription_api_should_return_bad_request_status_when_missing_data() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=jhon%20doe", "missing email"),
        ("email=jhondoe%40email.com", "missing name"),
        ("", "missing name and email"),
    ];

    // Act
    for (body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status().as_u16(),
            "{}",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");

    let port = listener.local_addr().unwrap().port();

    let server = run(listener).expect("Failed to bind address.");

    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
