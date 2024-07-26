mod authentication_service;
mod env_loader;
mod surreal_server;

use authentication_service::{
    authentication_service_server::AuthenticationServiceServer, AuthenticationService,
};
use std::env;
use tonic::transport::Server;

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();

    // Load the environment variables
    if let Err(e) = env_loader::load() {
        log::error!("Failed to load the environment variables: {}", e);
        return;
    }

    // Start a Surreal server
    let _surreal_handler = match surreal_server::start().await {
        Ok(handler) => handler,
        Err(e) => {
            log::error!("Failed to start the Surreal server: {}", e);
            return;
        }
    };

    // Create a new authentication service
    let mut auth_service = AuthenticationService::new("test", "auth");
    if let Err(e) = auth_service.start().await {
        log::error!("Failed to start the authentication service: {}", e);
        return;
    }

    // Get the RPC address
    let rpc_port = env::var("RPC_PORT").expect("RPC_PORT is not set");
    let addr = format!("0.0.0.0:{rpc_port}").parse().unwrap();

    // Start the authentication service on a separate thread
    log::info!("Server starting at: {}", addr);
    if let Err(e) = Server::builder()
        .add_service(AuthenticationServiceServer::new(auth_service))
        .serve(addr)
        .await
    {
        log::error!("Failed to start auth service: {}", e);
    };
}
