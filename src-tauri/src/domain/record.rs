use super::clock::Timestamp;
use super::tokens::TokenCounts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageEvent {
    pub at: Timestamp,
    pub model: String,
    pub project: String,
    pub tokens: TokenCounts,
}
