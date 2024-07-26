mod test;
mod authentication_service_proto {
    tonic::include_proto!("authentication_service_proto");
}

pub use authentication_service_proto::*;
use serde_json::Value;
use std::env;
use std::error::Error;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tonic::{Request, Response, Status};

/// The authentication service for a SurrealDB server
pub struct AuthenticationService {
    /// The namespace for the authentication database
    ns: String,
    /// The name of the authentication database
    db: String,
    /// The connected surreal session
    session: Option<Surreal<Client>>,
}

impl AuthenticationService {
    /// Create a new authentication service
    /// [ns] is the namespace containing the authentication database
    /// [db] is the name of the authentication database
    pub fn new(ns: &str, db: &str) -> Self {
        Self {
            ns: ns.to_string(),
            db: db.to_string(),
            session: None,
        }
    }

    /// Start the authentication service by connecting to the Surreal server
    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        // Get the username and password from the env
        let user = env::var("SURREAL_USER").expect("SURREAL_USER is not set");
        let pass = env::var("SURREAL_PASS").expect("SURREAL_PASS is not set");
        let port = env::var("SURREAL_PORT").expect("SURREAL_PORT is not set");

        // Connect to the Surreal server
        let address = format!("0.0.0.0:{}", port);
        let session = match Surreal::new::<Ws>(&address).await {
            Ok(session) => session,
            Err(e) => {
                log::error!("Failed to connect to the Surreal server: {}", e);
                return Err(e.into());
            }
        };

        // Sign in to the Surreal server
        if let Err(e) = session
            .signin(Root {
                username: &user,
                password: &pass,
            })
            .await
        {
            log::error!("Failed to sign in to the Surreal server: {}", e);
            return Err(e.into());
        }

        // Use the specified namespace and database
        if let Err(e) = session.use_ns(&self.ns).use_db(&self.db).await {
            log::error!("Failed to use the specified namespace and database: {}", e);
            return Err(e.into());
        }

        // Save the session
        self.session = Some(session);

        Ok(())
    }

    #[allow(dead_code)]
    /// Stop the authentication service and disconnect from the Surreal server
    pub fn stop(&mut self) {
        let _ = self.session.take();
    }
}

#[tonic::async_trait]
impl authentication_service_server::AuthenticationService for AuthenticationService {
    /// Create a new user account
    async fn create_user(
        &self,
        req: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        // Get the session
        let session = match self.session.as_ref() {
            Some(session) => session,
            None => {
                log::error!("Session has not been created");
                return Err(Status::internal("Session has not been created"));
            }
        };

        // Get the request
        let req = req.into_inner();

        // Get the user
        let result: Option<Value> = match session
            .query(format!("SELECT * FROM user:{0}", req.email))
            .await
        {
            Ok(mut result) => match result.take(0) {
                Ok(user) => user,
                Err(_) => {
                    return Err(Status::internal("Failed to check for the user"));
                }
            },
            Err(e) => {
                log::error!("Failed to get the user: {}", e);
                return Err(Status::internal("Failed to check for the user"));
            }
        };

        // Check if the user exists
        if result.is_some() {
            return Err(Status::already_exists("User already exists"));
        }

        // Create the user
        if let Err(e) = session
            .query(format!(
                "CREATE user:{0} SET name='{1}',password='{2}',email='{3}'",
                req.email, req.name, req.password, req.email
            ))
            .await
        {
            log::error!("Failed to create the user: {}", e);
            return Err(Status::internal("Failed to create the user"));
        }

        Ok(Response::new(CreateUserResponse {}))
    }

    /// Log in the user and return a jwt token
    async fn log_in(&self, req: Request<LogInRequest>) -> Result<Response<LogInResponse>, Status> {
        // Get the session
        let session = match self.session.as_ref() {
            Some(session) => session,
            None => {
                log::error!("Session has not been created");
                return Err(Status::internal("Session has not been created"));
            }
        };

        // Get the request
        let req = req.into_inner();

        // Get the password of the user
        let result: Option<Value> = match session
            .query(format!("SELECT password, email FROM user:{0} ", req.email))
            .await
        {
            Ok(mut result) => match result.take(0) {
                Ok(user) => user,
                Err(e) => {
                    log::error!("Failed to get the user: {}", e);
                    return Err(Status::internal("Failed to get the user"));
                }
            },
            Err(e) => {
                log::error!("Failed to get the user: {}", e);
                return Err(Status::internal("Failed to get the user"));
            }
        };

        // Extract the user
        let result = match result {
            Some(result) => result,
            None => {
                return Err(Status::not_found("User not found"));
            }
        };

        // Check the password
        if result["password"] != req.password {
            return Err(Status::unauthenticated("Incorrect password"));
        }

        Ok(Response::new(LogInResponse {
            token: result["email"].as_str().unwrap().to_string(),
        }))
    }

    /// Delete a user account
    async fn delete_user(
        &self,
        req: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        // Get the session
        let session = match self.session.as_ref() {
            Some(session) => session,
            None => {
                log::error!("Session has not been created");
                return Err(Status::internal("Session has not been created"));
            }
        };

        // Get the request
        let req = req.into_inner();

        // Get the user
        let result: Option<Value> = match session
            .query(format!("SELECT * FROM user:{0}", req.token))
            .await
        {
            Ok(mut result) => match result.take(0) {
                Ok(user) => user,
                Err(_) => {
                    return Err(Status::internal("Failed to get the user"));
                }
            },
            Err(e) => {
                log::error!("Failed to get the user: {}", e);
                return Err(Status::internal("Failed to get the user"));
            }
        };

        // Check if the user exists
        if result.is_none() {
            return Err(Status::not_found("User not found"));
        }

        // Delte the user
        if let Err(e) = session
            .query(format!("DELETE user:{0} RETURN NONE", req.token))
            .await
        {
            log::error!("Failed to delete the user: {}", e);
            return Err(Status::internal("Failed to delete the user"));
        }

        Ok(Response::new(DeleteUserResponse {}))
    }
}
