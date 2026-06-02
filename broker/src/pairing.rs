use uuid::Uuid;

use crate::session::SharedState;

pub async fn resolve_pairing_code(state: &SharedState, code: &str) -> Option<Uuid> {
    let guard = state.read().await;
    guard.pairing_codes.get(code).copied()
}
