use serde::Serialize;

use super::engine::Report;
use crate::domain::breakdown::{Breakdown, Breakdowns, Ranking};
use crate::domain::clock::Timestamp;
use crate::domain::history::{BUCKET, HourlyHistory};
use crate::domain::limit::LimitSnapshot;
use crate::domain::period::Window;
use crate::domain::summary::TokenWindow;
use crate::sources::oauth::status::OAuthStatus;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitView {
    pub kind: &'static str,
    pub percent: f64,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenView {
    pub tokens: u64,
    pub window_start: Option<i64>,
    pub window_end: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryView {
    pub start: i64,
    pub bucket_millis: i64,
    pub tokens: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareView {
    pub name: String,
    pub tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingView {
    pub top: Vec<ShareView>,
    pub other: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownView {
    pub models: RankingView,
    pub projects: RankingView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownsView {
    pub day: BreakdownView,
    pub week: BreakdownView,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageView {
    pub accurate_mode: bool,
    pub status: OAuthStatus,
    pub limits: Vec<LimitView>,
    pub five_hour: TokenView,
    pub weekly: TokenView,
    pub history: HistoryView,
    pub breakdown: BreakdownsView,
    pub generated_at: i64,
}

impl UsageView {
    #[must_use]
    pub fn from_report(report: &Report, now: Timestamp) -> Self {
        Self {
            accurate_mode: report.accurate_mode,
            status: report.status,
            limits: report.summary.limits.iter().map(limit_view).collect(),
            five_hour: token_view(report.summary.five_hour),
            weekly: token_view(report.summary.weekly),
            history: history_view(&report.summary.history),
            breakdown: breakdowns_view(&report.summary.breakdowns),
            generated_at: now.unix_millis(),
        }
    }
}

fn limit_view(limit: &LimitSnapshot) -> LimitView {
    LimitView {
        kind: limit.kind.name(),
        percent: limit.utilization.percent(),
        resets_at: limit.resets_at.map(Timestamp::unix_millis),
    }
}

fn breakdowns_view(breakdowns: &Breakdowns) -> BreakdownsView {
    BreakdownsView {
        day: breakdown_view(&breakdowns.day),
        week: breakdown_view(&breakdowns.week),
    }
}

fn breakdown_view(breakdown: &Breakdown) -> BreakdownView {
    BreakdownView {
        models: ranking_view(&breakdown.models),
        projects: ranking_view(&breakdown.projects),
    }
}

fn ranking_view(ranking: &Ranking) -> RankingView {
    RankingView {
        top: ranking
            .top
            .iter()
            .map(|share| ShareView {
                name: share.name.clone(),
                tokens: share.tokens,
            })
            .collect(),
        other: ranking.other,
    }
}

fn history_view(history: &HourlyHistory) -> HistoryView {
    HistoryView {
        start: history.start.unix_millis(),
        bucket_millis: BUCKET.millis(),
        tokens: history.tokens.clone(),
    }
}

fn token_view(window: TokenWindow) -> TokenView {
    TokenView {
        tokens: window.tokens.excluding_cache_reads(),
        window_start: window
            .window
            .map(|window| Window::start(window).unix_millis()),
        window_end: window
            .window
            .map(|window| Window::end(window).unix_millis()),
    }
}

#[cfg(test)]
mod tests {
    use super::UsageView;
    use crate::app::engine::Report;
    use crate::domain::breakdown::{Breakdown, Breakdowns, Ranking, Share};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::history::HourlyHistory;
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::domain::period::Window;
    use crate::domain::summary::{TokenWindow, UsageSummary};
    use crate::domain::tokens::TokenCounts;
    use crate::sources::oauth::status::OAuthStatus;
    use serde_json::json;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    #[test]
    fn serializes_camel_case_for_the_popup() {
        let window = Window::starting_at(NOW - Span::hours(1), Span::FIVE_HOURS);
        let report = Report {
            accurate_mode: true,
            status: OAuthStatus::Active,
            summary: UsageSummary {
                limits: vec![LimitSnapshot {
                    kind: LimitKind::FiveHour,
                    utilization: Utilization::from_percent(42.5).expect("valid percent"),
                    resets_at: Some(window.end()),
                }],
                five_hour: TokenWindow {
                    window: Some(window),
                    tokens: TokenCounts {
                        input: 1,
                        output: 2,
                        cache_creation: 3,
                        cache_read: 400,
                    },
                },
                weekly: TokenWindow {
                    window: None,
                    tokens: TokenCounts::default(),
                },
                history: HourlyHistory {
                    start: NOW,
                    tokens: vec![0, 7],
                },
                breakdowns: Breakdowns {
                    day: Breakdown {
                        models: Ranking {
                            top: vec![Share {
                                name: "claude-opus-5".to_owned(),
                                tokens: 6,
                            }],
                            other: 1,
                        },
                        projects: Ranking::default(),
                    },
                    week: Breakdown::default(),
                },
            },
        };
        let value = serde_json::to_value(UsageView::from_report(&report, NOW)).expect("json");
        assert_eq!(
            value,
            json!({
                "accurateMode": true,
                "status": "active",
                "limits": [{
                    "kind": "five_hour",
                    "percent": 42.5,
                    "resetsAt": window.end().unix_millis()
                }],
                "fiveHour": {
                    "tokens": 6,
                    "windowStart": window.start().unix_millis(),
                    "windowEnd": window.end().unix_millis()
                },
                "weekly": { "tokens": 0, "windowStart": null, "windowEnd": null },
                "history": {
                    "start": NOW.unix_millis(),
                    "bucketMillis": 3_600_000,
                    "tokens": [0, 7]
                },
                "breakdown": {
                    "day": {
                        "models": { "top": [{ "name": "claude-opus-5", "tokens": 6 }], "other": 1 },
                        "projects": { "top": [], "other": 0 }
                    },
                    "week": {
                        "models": { "top": [], "other": 0 },
                        "projects": { "top": [], "other": 0 }
                    }
                },
                "generatedAt": NOW.unix_millis()
            })
        );
    }
}
