#[cfg(test)]
mod test_serreal_server {
    use super::super::start;
    use std::env;

    #[tokio::test]
    async fn test_start_server() {
        env::set_var("SURREAL_USER", "root");
        env::set_var("SURREAL_PASS", "root");
        env::set_var("SURREAL_PORT", "9000");
        env::set_var("SURREAL_TARGET", "memory");

        // Start the Surreal server
        let _handler = start().await.unwrap();

        // Wait for 4 seconds
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
}
