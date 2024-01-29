#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use news_letter::configuration::get_configuration;
    use sqlx::PgPool;

    struct TestApp {
        pub address: String,
        pub db_pool: PgPool,
    }

    async fn spwan_app() -> TestApp {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to a random port");
        let port = listener.local_addr().unwrap().port();
        let configuration = get_configuration().expect("Failed to read configuration");
        let pg_pool = PgPool::connect(&configuration.database.connection_string())
            .await
            .expect("Failed to connecto to postgres.");

        let server = news_letter::startup::run(listener, pg_pool.clone())
            .expect("failed to bind to address");

        let _ = tokio::spawn(server);

        TestApp {
            address: format!("http://127.0.0.1:{}", port),
            db_pool: pg_pool.clone(),
        }
    }

    #[actix_web::test]
    async fn health_check_succeeds() {
        let test_app = spwan_app().await;

        let client = reqwest::Client::new();
        let res = client
            .get(&format!("{}/health_check", &test_app.address))
            .send()
            .await
            .expect("Failed to execute request");

        assert!(res.status().is_success());
        assert_eq!(Some(0), res.content_length());
    }

    #[actix_web::test]
    async fn subscribe_returns_a_200_for_valid_form_data() {
        let test_app = spwan_app().await;

        let client = reqwest::Client::new();
        let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

        let res = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute reqwest.");

        assert_eq!(200, res.status().as_u16());

        let saved = sqlx::query!("SELECT email, name FROM subscriptions")
            .fetch_one(&test_app.db_pool)
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(saved.email, "ursula_le_guin@gmail.com");
        assert_eq!(saved.name, "le guin");
    }

    #[actix_web::test]
    async fn subscribe_returns_400_when_data_is_missing() {
        let test_app = spwan_app().await;
        let client = reqwest::Client::new();
        let test_cases = vec![
            ("name=le%20guin", "missing the email"),
            ("email=ursula_le_guin%40gmail.com", "missing the name"),
            ("", "missing both name and email"),
        ];

        for (body, err_msg) in test_cases {
            let rsp = client
                .post(&format!("{}/subscriptions", &test_app.address))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body)
                .send()
                .await
                .expect("Failed to execute reqwest");

            assert_eq!(
                400,
                rsp.status().as_u16(),
                "The api didnot fail with 400 when the payload was: {}",
                err_msg
            );
        }
    }
}
