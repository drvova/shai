use axum::{
    response::sse::Event,
    routing::post,
    Router,
};
use futures::stream::Stream;
use shai_core::agent::{AgentBuilder, AgentError};
use std::convert::Infallible;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use uuid::Uuid;

pub mod apis;
pub mod error;

pub use error::{ApiJson, ErrorResponse};

/// Server state (currently empty, can be extended with shared resources)
#[derive(Clone)]
pub struct ServerState {}

/// Helper to create an agent with proper error handling
/// Returns appropriate error responses based on the error type
pub async fn create_agent_from_model(
    model: &str,
    session_id: &Uuid,
) -> Result<AgentBuilder, ErrorResponse> {
    // Use the model field to select the agent config
    // If model is "default" or empty, use None to load default agent
    let agent_config_name = if model.is_empty() || model == "default" {
        None
    } else {
        Some(model.to_string())
    };

    AgentBuilder::create(agent_config_name).await.map_err(|e| {
        match e {
            AgentError::ConfigurationError(msg) => {
                // Check if it's a "does not exist" error
                if msg.contains("does not exist") {
                    error!("[{}] Agent not found: {}", session_id, msg);
                    ErrorResponse::not_found(format!("Agent '{}' not found", model))
                } else {
                    error!("[{}] Configuration error: {}", session_id, msg);
                    ErrorResponse::invalid_request(msg)
                }
            }
            _ => {
                error!("[{}] Failed to create agent: {}", session_id, e);
                ErrorResponse::internal_error(format!("Failed to create agent: {}", e))
            }
        }
    })
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
    addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = ServerState {};

    let app = Router::new()
        // Simple API
        .route("/v1/multimodal", post(apis::simple::handle_multimodal_query_stream))
        // OpenAI-compatible APIs
        .route("/v1/chat/completions", post(apis::openai::handle_chat_completion))
        .route("/v1/responses", post(apis::openai::handle_response))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Print server info
    println!("Server starting on \x1b[1mhttp://{}\x1b[0m", addr);
    println!("\nAvailable endpoints:");
    println!("  \x1b[1mPOST /v1/chat/completions\x1b[0m    - OpenAI-compatible chat completion API");
    println!("  \x1b[1mPOST /v1/responses\x1b[0m           - OpenAI-compatible responses API (stateless)");
    println!("  \x1b[1mPOST /v1/multimodal\x1b[0m          - Multimodal query API (streaming)");

    // List available agents
    use shai_core::config::agent::AgentConfig;
    match AgentConfig::list_agents() {
        Ok(agents) if !agents.is_empty() => {
            println!("\nAvailable agents: \x1b[2m{}\x1b[0m", agents.join(", "));
        }
        _ => {}
    }

    println!("\nPress Ctrl+C to stop\n");

    info!("HTTP server listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}