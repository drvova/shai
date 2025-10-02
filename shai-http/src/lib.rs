use axum::{
    response::sse::Event,
    routing::post,
    Router,
};
use futures::stream::Stream;
use shai_core::agent::AgentBuilder;
use std::convert::Infallible;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use uuid::Uuid;

pub mod apis;

/// Function that creates a new agent for each request
/// Takes message_history and returns an AgentBuilder
pub type AgentFactory = Arc<dyn Fn(Vec<shai_llm::ChatMessage>) -> AgentBuilder + Send + Sync>;

/// Server state containing the agent factory
#[derive(Clone)]
pub struct ServerState {
    pub agent_factory: AgentFactory,
}

/// Stream wrapper that detects client disconnection
pub struct DisconnectionHandler {
    pub stream: std::pin::Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>,
    pub controller: Option<shai_core::agent::AgentController>,
    pub session_id: Uuid,
    pub completed: bool,
}

impl Stream for DisconnectionHandler {
    type Item = Result<Event, Infallible>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.stream.as_mut().poll_next(cx) {
            std::task::Poll::Ready(None) => {
                // Stream ended normally
                self.completed = true;
                std::task::Poll::Ready(None)
            }
            other => other,
        }
    }
}

impl Drop for DisconnectionHandler {
    fn drop(&mut self) {
        if let Some(controller) = self.controller.take() {
            let session_id = self.session_id;
            if self.completed {
                info!("[{}] Stream completed normally", session_id);
            } else {
                info!("[{}] Client disconnected - cancelling agent", session_id);
                tokio::spawn(async move {
                    let _ = controller.cancel().await;
                });
            }
        }
    }
}

/// Start the HTTP server with SSE streaming
pub async fn start_server(
    agent_factory: AgentFactory,
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = ServerState { agent_factory };

    let app = Router::new()
        // Simple API
        .route("/v1/multimodal", post(apis::simple::handle_multimodal_query_stream))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("HTTP server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}