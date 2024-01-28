#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    async fn spwan_app() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to a random port");
        let port = listener.local_addr().unwrap().port();
        let server = news_letter::run(listener).expect("failed to bind to address");

        let _ = tokio::spawn(server);

        format!("http://127.0.0.1:{}", port)
    }

    #[actix_web::test]
    async fn health_check_succeeds() {
        let address = spwan_app().await;

        let client = reqwest::Client::new();
        let res = client
            .get(&format!("{}/health_check", &address))
            .send()
            .await
            .expect("Failed to execute request");

        assert!(res.status().is_success());
        assert_eq!(Some(0), res.content_length());
    }
}
