#[cfg(test)]
mod authenticaion_service_test {

    use super::super::super::surreal_server;
    use super::super::{
        authentication_service_client::AuthenticationServiceClient,
        authentication_service_server::AuthenticationServiceServer, *,
    };
    use std::env;
    use tonic::transport::{Channel, Server};

    #[tokio::test]
    async fn full_test() {
        env::set_var("SURREAL_USER", "root");
        env::set_var("SURREAL_PASS", "root");
        env::set_var("SURREAL_PORT", "9000");
        env::set_var("SURREAL_TARGET", "memory");

        // Start the Surreal server
        let _surreal_handler = surreal_server::start().await.unwrap();

        // Create a new authentication service
        let mut auth_service = AuthenticationService::new("test", "auth");
        auth_service.start().await.unwrap();

        // Start the rpc server
        let service_handler = tokio::spawn(async move {
            let addr = "0.0.0.0:50051".parse().unwrap();
            Server::builder()
                .add_service(AuthenticationServiceServer::new(auth_service))
                .serve(addr)
                .await
                .unwrap();
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        //Create a client to connect to the server
        let auth_channel = Channel::from_static("http://0.0.0.0:50051")
            .connect()
            .await
            .unwrap();
        let mut auth_client = AuthenticationServiceClient::new(auth_channel);

        // Create a new user
        auth_client
            .create_user(CreateUserRequest {
                name: "test_name".to_string(),
                email: "test_email".to_string(),
                password: "test_password".to_string(),
            })
            .await
            .unwrap();

        // Log in to that user
        let token = auth_client
            .log_in(LogInRequest {
                email: "test_email".to_string(),
                password: "test_password".to_string(),
            })
            .await
            .unwrap()
            .into_inner()
            .token;
        assert!(!token.is_empty());
        assert_eq!(token, "test_email");

        // Log in to a user that doesn't exist
        auth_client
            .log_in(LogInRequest {
                email: "none".to_string(),
                password: "none".to_string(),
            })
            .await
            .unwrap_err();

        // Log in with a wrong password
        auth_client
            .log_in(LogInRequest {
                email: "test_email".to_string(),
                password: "wrong".to_string(),
            })
            .await
            .unwrap_err();

        // Delete the user
        auth_client
            .delete_user(DeleteUserRequest { token })
            .await
            .unwrap();

        // Log in to that user again after deletion
        auth_client
            .log_in(LogInRequest {
                email: "test_email".to_string(),
                password: "test_password".to_string(),
            })
            .await
            .unwrap_err();

        // Stop the surreal server and the rpc server
        service_handler.abort();
    }
}
