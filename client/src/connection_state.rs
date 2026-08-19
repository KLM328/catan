use std::time::Instant;

#[derive(Clone, Copy)]
pub(crate) enum ConnectionState {
    NotConnected,
    Connecting { attempt: u32 },
    Connected,
    Retrying { at: Instant, attempt: u32 },
}