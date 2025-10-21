use shai_core::agent::AgentController;
use tokio::sync::OwnedMutexGuard;
use tracing::info;

use crate::session::logger::colored_session_id;


pub enum RequestLifecycle {
    Background {
        controller_guard: OwnedMutexGuard<AgentController>,
        request_id: String,
        session_id: String,
    },
    Ephemeral {
        controller_guard: OwnedMutexGuard<AgentController>,
        request_id: String,
        session_id: String,
    },
}

impl RequestLifecycle {
    pub fn new(ephemeral: bool, controller_guard: OwnedMutexGuard<AgentController>, request_id: String, session_id: String) -> Self {
        match ephemeral {
            true => Self::Ephemeral { controller_guard, request_id, session_id },
            false => Self::Background { controller_guard, request_id, session_id },
        }
    }
}

impl Drop for RequestLifecycle {
    fn drop(&mut self) {
        match self {
            Self::Background { request_id, session_id, .. } => {
                info!(
                    "[{}] - {} Stream completed, releasing controller lock (background session)",
                    request_id,
                    colored_session_id(session_id)
                );
            }
            Self::Ephemeral { controller_guard, request_id, session_id } => {
                info!(
                    "[{}] - {} Stream completed, destroying agent (ephemeral session)",
                    request_id,
                    colored_session_id(session_id)
                );

                // Clone before moving into async task
                let ctrl = controller_guard.clone();
                tokio::spawn(async move {
                    let _ = ctrl.terminate().await;
                });
            }
        }
    }
}
