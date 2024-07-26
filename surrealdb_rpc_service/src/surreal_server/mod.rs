mod drop_dead_handler;
mod test;

use drop_dead_handler::DropDeadHandler;
use std::env;
use std::error::Error;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

/// Start the Surreal server.
/// Returns the handler used to kill the server later.
/// This will wait for 1 second to ensure the server is up before returning but there's
/// no guarantee that the server is actually up.
pub async fn start() -> Result<DropDeadHandler, Box<dyn Error>> {
    let user = env::var("SURREAL_USER").expect("SURREAL_USER is not set");
    let pass = env::var("SURREAL_PASS").expect("SURREAL_PASS is not set");
    let port = env::var("SURREAL_PORT").expect("SURREAL_PORT is not set");
    let target = env::var("SURREAL_TARGET").expect("SURREAL_TARGET is not set");
    let handler = match Command::new("surreal")
        .arg("start")
        .arg(target)
        .arg("--auth")
        .args(["-u", &user])
        .args(["-p", &pass])
        .arg("--no-banner")
        .args(["--bind", &format!("0.0.0.0:{port}")])
        .spawn()
    {
        Ok(output) => output,
        Err(e) => return Err(e.into()),
    };
    sleep(Duration::from_secs(1)).await;

    // Wrap the handler in a DropDeadHandler
    let handler = DropDeadHandler::new(handler);

    Ok(handler)
}
