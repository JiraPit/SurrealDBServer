use tokio::process::Child;

/// A handler that kills the Surreal server when dropped.
pub struct DropDeadHandler {
    handler: Child,
}

impl DropDeadHandler {
    pub fn new(handler: Child) -> Self {
        Self { handler }
    }
}

impl Drop for DropDeadHandler {
    fn drop(&mut self) {
        self.handler.start_kill().unwrap_or(());
    }
}
