mod authentication_service;
mod env_loader;
mod surreal_server;

use authentication_service::{
    authentication_service_server::AuthenticationServiceServer, AuthenticationService,
};
use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    routing::any,
    Router,
};
use hyper::client::conn::http1;
use hyper::client::conn::http2;
use hyper_tls::HttpsConnector;
use std::{env, error::Error};
use tokio::time::{sleep, Duration};
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
    let surreal_handler = match surreal_server::start().await {
        Ok(handler) => handler,
        Err(e) => {
            log::error!("Failed to start the Surreal server: {}", e);
            return;
        }
    };

    // Start an authentication service on a separate thread
    let auth_service_handler = match spawn_auth_service().await {
        Ok(handler) => handler,
        Err(e) => {
            log::error!("Failed to start the authentication service: {}", e);
            return;
        }
    };

    // Start the router and the listener
    let router = Router::new()
        .route("/interface", any(handle_interface_route))
        .route("/data", any(handle_data_route));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Error binding server");

    // server the server
    axum::serve(listener, router)
        .await
        .expect("Error running server");

    // Explicitly stop the services and drop the handlers
    auth_service_handler.abort();
    while !auth_service_handler.is_finished() {
        sleep(Duration::from_millis(300)).await;
    }
    drop(surreal_handler);
    drop(auth_service_handler);
}

/// Spawns an authentication service on a separate thread
async fn spawn_auth_service() -> Result<tokio::task::JoinHandle<()>, Box<dyn Error>> {
    // Create a new authentication service
    let mut auth_service = AuthenticationService::new("test", "auth");
    auth_service.start().await?;

    // Get the RPC address
    let rpc_port = env::var("RPC_PORT").expect("RPC_PORT is not set");
    let addr = format!("0.0.0.0:{rpc_port}").parse().unwrap();

    // Start the authentication service on a separate thread
    let handler = tokio::spawn(async move {
        // Start the authentication service
        log::info!("Server starting at: {}", addr);
        if let Err(e) = Server::builder()
            .add_service(AuthenticationServiceServer::new(auth_service))
            .serve(addr)
            .await
        {
            log::error!("Failed to start auth service: {}", e);
        }
    });

    Ok(handler)
}

/// Handles the interface route
async fn handle_interface_route(req: Request<Body>) -> Result<Response<Body>, StatusCode> {
    todo!()
}

/// Handles the data route
async fn handle_data_route(req: Request<Body>) -> Result<Response<Body>, StatusCode> {
    todo!()
}
