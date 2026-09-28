use serde::Serialize;

use super::engine::Report;
use crate::domain::alerts::Alert;
use crate::domain::breakdown::{Breakdown, Breakdowns, Ranking};
use crate::domain::calibration::{Basis, Estimate};
use crate::domain::clock::Timestamp;
use crate::domain::history::{BUCKET, HourlyHistory};
use crate::domain::limit::LimitSnapshot;
use crate::domain::period::Window;
use crate::domain::plan::Plan;
use crate::domain::preferences::{Language, PollInterval, Preferences};
use crate::domain::refresh::ManualRefresh;
use crate::domain::summary::TokenWindow;
use crate::sources::oauth::status::OAuthStatus;
use crate::toast::Message;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitView {
    pub kind: &'static str,
    pub percent: f64,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateView {
    pub kind: &'static str,
    pub percent: f64,
    pub source: &'static str,
    pub samples: usize,
    pub entered_percent: Option<f64>,
    pub entered_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualView {
    pub kind: &'static str,
    pub percent: f64,
    pub entered_at: i64,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdsView {
    pub elevated: u8,
    pub high: u8,
    pub critical: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesView {
    pub thresholds: ThresholdsView,
    pub poll_minutes: u8,
    pub poll_choices: Vec<u8>,
    pub language: &'static str,
    pub resolved_language: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshView {
    pub state: &'static str,
    pub ready_at: Option<i64>,
}

impl RefreshView {
    #[must_use]
    pub const fn new(refresh: ManualRefresh) -> Self {
        match refresh {
            ManualRefresh::Ready => Self {
                state: "ready",
                ready_at: None,
            },
            ManualRefresh::Running => Self {
                state: "running",
                ready_at: None,
            },
            ManualRefresh::CoolingDown { until } => Self {
                state: "cooling",
                ready_at: Some(until.unix_millis()),
            },
            ManualRefresh::Blocked => Self {
                state: "blocked",
                ready_at: None,
            },
        }
    }
}

#[must_use]
pub const fn login_state(status: OAuthStatus, accurate_mode: bool, running: bool) -> &'static str {
    if running {
        return "running";
    }
    match status {
        OAuthStatus::SignedOut | OAuthStatus::TokenExpired if accurate_mode => "ready",
        _ => "hidden",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewContext {
    pub language: Language,
    pub autostart: bool,
    pub refresh: ManualRefresh,
    pub plan: Option<Plan>,
    pub login_running: bool,
    pub now: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertView {
    pub kind: &'static str,
    pub severity: &'static str,
    pub title: String,
    pub body: String,
}

impl AlertView {
    #[must_use]
    pub fn new(alert: &Alert, message: Message) -> Self {
        Self {
            kind: alert.kind.name(),
            severity: alert.severity.name(),
            title: message.title,
            body: message.body,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageView {
    pub accurate_mode: bool,
    pub status: OAuthStatus,
    pub official_updated_at: Option<i64>,
    pub preferences: PreferencesView,
    pub limits: Vec<LimitView>,
    pub estimates: Vec<EstimateView>,
    pub manual: Vec<ManualView>,
    pub five_hour: TokenView,
    pub weekly: TokenView,
    pub history: HistoryView,
    pub breakdown: BreakdownsView,
    pub autostart: bool,
    pub refresh: RefreshView,
    pub plan: Option<String>,
    pub login: &'static str,
    pub generated_at: i64,
}

impl UsageView {
    #[must_use]
    pub fn from_report(report: &Report, context: ViewContext) -> Self {
        let ViewContext {
            language,
            autostart,
            refresh,
            plan,
            login_running,
            now,
        } = context;
        Self {
            accurate_mode: report.accurate_mode,
            status: report.status,
            official_updated_at: report.official_updated_at.map(Timestamp::unix_millis),
            preferences: preferences_view(report.preferences, language),
            limits: report.summary.limits.iter().map(limit_view).collect(),
            estimates: report.estimates.iter().map(estimate_view).collect(),
            manual: report
                .manual
                .iter()
                .map(|reading| ManualView {
                    kind: reading.kind.name(),
                    percent: reading.utilization.percent(),
                    entered_at: reading.entered_at.unix_millis(),
                })
                .collect(),
            five_hour: token_view(report.summary.five_hour),
            weekly: token_view(report.summary.weekly),
            history: history_view(&report.summary.history),
            breakdown: breakdowns_view(&report.summary.breakdowns),
            autostart,
            refresh: RefreshView::new(refresh),
            plan: plan.filter(|_| report.accurate_mode).map(Plan::label),
            login: login_state(report.status, report.accurate_mode, login_running),
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

fn preferences_view(preferences: Preferences, language: Language) -> PreferencesView {
    let thresholds = preferences.thresholds;
    PreferencesView {
        thresholds: ThresholdsView {
            elevated: thresholds.elevated(),
            high: thresholds.high(),
            critical: thresholds.critical(),
        },
        poll_minutes: preferences.poll_interval.minutes(),
        poll_choices: PollInterval::CHOICES.to_vec(),
        language: preferences.language.code(),
        resolved_language: language.code(),
    }
}

fn estimate_view(estimate: &Estimate) -> EstimateView {
    let (source, samples, entered_percent, entered_at) = match estimate.basis {
        Basis::Calibrated { samples } => ("calibration", samples, None, None),
        Basis::Manual {
            entered_at,
            entered,
        } => (
            "manual",
            0,
            Some(entered.percent()),
            Some(entered_at.unix_millis()),
        ),
    };
    EstimateView {
        kind: estimate.kind.name(),
        percent: estimate.utilization.percent(),
        source,
        samples,
        entered_percent,
        entered_at,
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
    use super::{RefreshView, UsageView, ViewContext, login_state};
    use crate::app::engine::Report;
    use crate::domain::breakdown::{Breakdown, Breakdowns, Ranking, Share};
    use crate::domain::calibration::{Basis, Estimate};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::history::HourlyHistory;
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::domain::period::Window;
    use crate::domain::plan::{Plan, PlanKind};
    use crate::domain::preferences::{Language, Preferences};
    use crate::domain::refresh::ManualRefresh;
    use crate::domain::summary::{TokenWindow, UsageSummary};
    use crate::domain::tokens::TokenCounts;
    use crate::sources::oauth::status::OAuthStatus;
    use serde_json::json;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn sample_report(window: Window) -> Report {
        Report {
            accurate_mode: true,
            status: OAuthStatus::Active,
            official_updated_at: Some(NOW - Span::minutes(3)),
            preferences: Preferences::default(),
            manual: Vec::new(),
            estimates: vec![Estimate {
                kind: LimitKind::SevenDay,
                utilization: Utilization::from_percent(12.5).expect("valid percent"),
                basis: Basis::Calibrated { samples: 4 },
            }],
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
        }
    }

    #[test]
    fn serializes_camel_case_for_the_popup() {
        let window = Window::starting_at(NOW - Span::hours(1), Span::FIVE_HOURS);
        let report = sample_report(window);
        let value = serde_json::to_value(UsageView::from_report(
            &report,
            ViewContext {
                language: Language::Indonesian,
                autostart: true,
                refresh: ManualRefresh::CoolingDown {
                    until: NOW + Span::minutes(1),
                },
                plan: Some(Plan {
                    kind: PlanKind::Max,
                    multiplier: Some(5),
                }),
                login_running: false,
                now: NOW,
            },
        ))
        .expect("json");
        assert_eq!(
            value,
            json!({
                "accurateMode": true,
                "status": "active",
                "officialUpdatedAt": (NOW - Span::minutes(3)).unix_millis(),
                "preferences": {
                    "thresholds": { "elevated": 50, "high": 80, "critical": 95 },
                    "pollMinutes": 0,
                    "pollChoices": [0, 1, 2, 5, 10],
                    "language": "system",
                    "resolvedLanguage": "id"
                },
                "limits": [{
                    "kind": "five_hour",
                    "percent": 42.5,
                    "resetsAt": window.end().unix_millis()
                }],
                "estimates": [{
                    "kind": "seven_day",
                    "percent": 12.5,
                    "source": "calibration",
                    "samples": 4,
                    "enteredPercent": null,
                    "enteredAt": null
                }],
                "manual": [],
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
                "autostart": true,
                "refresh": {
                    "state": "cooling",
                    "readyAt": (NOW + Span::minutes(1)).unix_millis()
                },
                "plan": "Max 5x",
                "login": "hidden",
                "generatedAt": NOW.unix_millis()
            })
        );
    }

    #[test]
    fn the_plan_is_hidden_outside_accurate_mode() {
        let window = Window::starting_at(NOW - Span::hours(1), Span::FIVE_HOURS);
        let report = Report {
            accurate_mode: false,
            ..sample_report(window)
        };
        let view = UsageView::from_report(
            &report,
            ViewContext {
                language: Language::English,
                autostart: false,
                refresh: ManualRefresh::Blocked,
                plan: Some(Plan {
                    kind: PlanKind::Pro,
                    multiplier: None,
                }),
                login_running: false,
                now: NOW,
            },
        );
        assert_eq!(view.plan, None);
    }

    #[test]
    fn login_is_offered_only_when_claude_code_needs_it() {
        assert_eq!(login_state(OAuthStatus::SignedOut, true, false), "ready");
        assert_eq!(login_state(OAuthStatus::TokenExpired, true, false), "ready");
        assert_eq!(login_state(OAuthStatus::SignedOut, false, false), "hidden");
        assert_eq!(login_state(OAuthStatus::Retrying, true, false), "hidden");
        assert_eq!(login_state(OAuthStatus::Active, true, true), "running");
    }

    #[test]
    fn refresh_states_have_stable_names() {
        assert_eq!(RefreshView::new(ManualRefresh::Ready).state, "ready");
        assert_eq!(RefreshView::new(ManualRefresh::Running).state, "running");
        assert_eq!(RefreshView::new(ManualRefresh::Blocked).state, "blocked");
        assert_eq!(RefreshView::new(ManualRefresh::Blocked).ready_at, None);
    }
}
