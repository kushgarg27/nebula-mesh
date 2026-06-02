use protocol::{Edges, EnterRemote, ExitRemote, Payload};

#[derive(Debug, Clone, Default)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn detect_edge_transition(pos: &CursorPosition, edges: &Edges, deadzone: i32) -> Option<Payload> {
    if pos.x <= deadzone {
        if let Some(target) = &edges.left {
            return Some(Payload::EnterRemote(EnterRemote {
                target_device_id: target.clone(),
            }));
        }
    }
    if pos.x >= pos.width - deadzone {
        if let Some(target) = &edges.right {
            return Some(Payload::EnterRemote(EnterRemote {
                target_device_id: target.clone(),
            }));
        }
    }
    if pos.y <= deadzone {
        if let Some(target) = &edges.top {
            return Some(Payload::EnterRemote(EnterRemote {
                target_device_id: target.clone(),
            }));
        }
    }
    if pos.y >= pos.height - deadzone {
        if let Some(target) = &edges.bottom {
            return Some(Payload::EnterRemote(EnterRemote {
                target_device_id: target.clone(),
            }));
        }
    }
    None
}

pub fn exit_remote() -> Payload {
    Payload::ExitRemote(ExitRemote {})
}
