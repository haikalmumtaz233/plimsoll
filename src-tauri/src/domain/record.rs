use super::clock::Timestamp;
use super::tokens::TokenCounts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageEvent {
    pub at: Timestamp,
    pub model: String,
    pub project: String,
    pub tokens: TokenCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventKey {
    pub message_id: String,
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedEvent {
    pub key: EventKey,
    pub event: UsageEvent,
}
