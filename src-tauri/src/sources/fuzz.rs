use std::io::{BufReader, Cursor};

use super::cli::usage_text;
use super::jsonl::line;
use super::jsonl::reader::{LineRead, read_line_bounded};
use super::oauth::credentials::{parse_access_token, parse_plan};
use super::oauth::response;
use super::rfc3339;
use crate::domain::clock::{Span, Timestamp};
use crate::domain::limit::LimitKind;

const ROUNDS: usize = 20_000;
const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);
const EARLIEST_RFC3339: i64 = -62_167_219_200_000;
const LATEST_RFC3339: i64 = 253_402_300_799_999;

const INTERESTING: &[&[u8]] = &[
    b"\"",
    b"{",
    b"}",
    b"[",
    b"]",
    b":",
    b",",
    b"\\",
    b"\\u0000",
    b"\\ud800",
    b"\n",
    b"\r\n",
    b"\xff\xfe",
    b"null",
    b"true",
    b"-1",
    b"1e309",
    b"-0.0",
    b"NaN",
    b"18446744073709551615",
    b"18446744073709551616",
    b"9223372036854775807",
    b"\"usage\"",
    b"\"assistant\"",
    b"\"<synthetic>\"",
    b"0000-01-01T00:00:00Z",
    b"9999-12-31T23:59:59.999999999Z",
    b"2024-02-30T00:00:00Z",
    b"T",
    b"Z",
    b".",
];

const JSONL_SEEDS: &[&[u8]] = &[
    br#"{"type":"assistant","timestamp":"2026-09-25T01:35:37.212Z","requestId":"req_1","cwd":"C:\\Users\\dev\\plimsoll","message":{"id":"msg_1","model":"claude-opus-5","role":"assistant","content":[{"type":"text","text":"hello"}],"usage":{"input_tokens":3,"output_tokens":40,"cache_creation_input_tokens":500,"cache_read_input_tokens":6000,"service_tier":"standard"}}}"#,
    br#"{"type":"assistant","timestamp":"2026-09-25T01:35:37+00:00","message":{"id":"msg_2","model":"claude-sonnet-5","usage":{"output_tokens":1}}}"#,
    br#"{"type":"user","timestamp":"2026-09-25T01:35:37.212Z","message":{"role":"user","content":"usage"}}"#,
    br#"{"type":"summary","summary":"x","leafUuid":"y"}"#,
];

const TIMESTAMP_SEEDS: &[&[u8]] = &[
    b"2026-09-25T01:35:37.212Z",
    b"2024-02-29T23:59:59+00:00",
    b"1970-01-01T00:00:00.123456789Z",
];

const OAUTH_SEEDS: &[&[u8]] = &[
    br#"{"limits":[{"kind":"session","percent":42.5,"resets_at":"2026-09-26T05:30:00Z"},{"kind":"weekly_all","percent":12,"resets_at":null}]}"#,
    br#"{"five_hour":{"utilization":81.0,"resets_at":"2026-09-26T05:30:00.000000+00:00"},"seven_day":{"utilization":3}}"#,
];

const USAGE_TEXT_SEEDS: &[&[u8]] = &[
    "Current session: 31% used \u{b7} resets Sep 29, 1:59am (Asia/Jakarta)\nCurrent week (all models): 12.5% used \u{b7} resets Oct 5, 6:59am (Asia/Jakarta)\n".as_bytes(),
    b"\x1b[1mCurrent session\x1b[22m: 74% left \xc2\xb7 resets 11:30pm (UTC)\r\nCurrent week: 9% used \xc2\xb7 resets Jan 2 at 7am (UTC)",
];

const CREDENTIAL_SEEDS: &[&[u8]] = &[
    br#"{"claudeAiOauth":{"accessToken":"fuzz-value","expiresAt":1790400000000,"scopes":["user:inference"]}}"#,
    br#"{"claudeAiOauth":{"accessToken":"fuzz-value","expiresAt":null}}"#,
    br#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
];

#[derive(Debug)]
struct Mutator {
    state: u64,
}

impl Mutator {
    const fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound.max(1)).unwrap_or(u64::MAX);
        usize::try_from(self.next() % bound).unwrap_or(0)
    }

    fn byte(&mut self) -> u8 {
        self.next().to_le_bytes()[0]
    }

    fn bytes(&mut self, length: usize) -> Vec<u8> {
        (0..length).map(|_| self.byte()).collect()
    }

    fn mutate(&mut self, seed: &[u8]) -> Vec<u8> {
        let mut data = seed.to_vec();
        for _ in 0..=self.below(4) {
            self.apply(&mut data);
        }
        data
    }

    fn apply(&mut self, data: &mut Vec<u8>) {
        let position = self.below(data.len() + 1);
        match self.below(6) {
            0 if !data.is_empty() => {
                let index = position.min(data.len() - 1);
                data[index] ^= 1 << self.below(8);
            }
            1 if !data.is_empty() => {
                let index = position.min(data.len() - 1);
                data[index] = self.byte();
            }
            2 => {
                let token = INTERESTING[self.below(INTERESTING.len())];
                data.splice(position..position, token.iter().copied());
            }
            3 => {
                let end = (position + self.below(16)).min(data.len());
                data.drain(position..end);
            }
            4 => {
                let end = (position + self.below(32)).min(data.len());
                let copy = data[position..end].to_vec();
                data.splice(position..position, copy);
            }
            _ => data.truncate(position),
        }
    }

    fn input(&mut self, seeds: &[&[u8]]) -> Vec<u8> {
        if self.below(10) == 0 {
            let length = self.below(256);
            self.bytes(length)
        } else {
            let seed = seeds[self.below(seeds.len())];
            self.mutate(seed)
        }
    }
}

#[test]
fn jsonl_lines_never_panic_and_only_yield_complete_events() {
    let mut mutator = Mutator::new(0x5eed_0001);
    let mut parsed = 0;
    for _ in 0..ROUNDS {
        let input = mutator.input(JSONL_SEEDS);
        if let Some(keyed) = line::parse(&input) {
            parsed += 1;
            assert_ne!(keyed.event.model, "");
            assert!(!keyed.event.model.starts_with('<'));
            assert_ne!(keyed.event.project, "");
            let millis = keyed.event.at.unix_millis();
            assert!((EARLIEST_RFC3339..=LATEST_RFC3339).contains(&millis));
        }
    }
    assert!(parsed > 0);
}

#[test]
fn timestamps_never_panic_and_stay_in_range() {
    let mut mutator = Mutator::new(0x5eed_0002);
    let mut parsed = 0;
    for _ in 0..ROUNDS {
        let input = mutator.input(TIMESTAMP_SEEDS);
        let Ok(text) = std::str::from_utf8(&input) else {
            continue;
        };
        if let Some(at) = rfc3339::parse(text) {
            parsed += 1;
            assert!((EARLIEST_RFC3339..=LATEST_RFC3339).contains(&at.unix_millis()));
        }
    }
    assert!(parsed > 0);
}

#[test]
fn oauth_bodies_never_panic_and_yield_valid_limits() {
    let mut mutator = Mutator::new(0x5eed_0003);
    let mut parsed = 0;
    for _ in 0..ROUNDS {
        let input = mutator.input(OAUTH_SEEDS);
        if let Some(snapshots) = response::parse(&input) {
            parsed += 1;
            assert!(!snapshots.is_empty() && snapshots.len() <= LimitKind::ALL.len());
            for snapshot in &snapshots {
                let percent = snapshot.utilization.percent();
                assert!(percent.is_finite() && percent >= 0.0);
            }
            let first = snapshots[0].kind;
            assert!(snapshots.iter().skip(1).all(|other| other.kind != first));
        }
    }
    assert!(parsed > 0);
}

#[test]
fn credentials_never_panic_and_never_yield_an_empty_token() {
    let mut mutator = Mutator::new(0x5eed_0004);
    let mut parsed = 0;
    for _ in 0..ROUNDS {
        let input = mutator.input(CREDENTIAL_SEEDS);
        if let Ok(token) = parse_access_token(&input, NOW) {
            parsed += 1;
            assert!(!token.secret().is_empty(), "parsed an empty token");
        }
    }
    assert!(parsed > 0);
}

#[test]
fn usage_text_never_panics_and_yields_valid_limits() {
    let mut mutator = Mutator::new(0x5eed_0007);
    let mut parsed = 0;
    for round in 0..ROUNDS {
        let input = mutator.input(USAGE_TEXT_SEEDS);
        let text = String::from_utf8_lossy(&input);
        let offset = Span::minutes(i64::try_from(round % 1_681).unwrap_or(0) - 840);
        let snapshots = usage_text::parse(&text, NOW, offset);
        parsed += usize::from(!snapshots.is_empty());
        assert!(snapshots.len() <= 2);
        for snapshot in snapshots {
            assert!((0.0..=100.0).contains(&snapshot.utilization.percent()));
            if let Some(resets_at) = snapshot.resets_at {
                assert!(resets_at > NOW - Span::days(2));
                assert!(resets_at < NOW + Span::days(400));
            }
        }
    }
    assert!(parsed > 0);
}

#[test]
fn plans_never_panic_and_always_have_a_short_label() {
    let mut mutator = Mutator::new(0x5eed_0006);
    let mut parsed = 0;
    for _ in 0..ROUNDS {
        let input = mutator.input(CREDENTIAL_SEEDS);
        if let Ok(Some(plan)) = parse_plan(&input) {
            parsed += 1;
            assert!(plan.label().len() <= "Enterprise 255x".len());
        }
    }
    assert!(parsed > 0);
}

#[test]
fn bounded_reader_accounts_for_every_complete_line() {
    let mut mutator = Mutator::new(0x5eed_0005);
    for _ in 0..ROUNDS / 10 {
        let length = mutator.below(512);
        let newline_every = 1 + mutator.below(64);
        let input: Vec<u8> = (0..length)
            .map(|_| {
                if mutator.below(newline_every) == 0 {
                    b'\n'
                } else {
                    mutator.byte()
                }
            })
            .collect();
        let limit = 1 + mutator.below(96);
        let capacity = 1 + mutator.below(16);
        let mut reader = BufReader::with_capacity(capacity, Cursor::new(input.clone()));
        let mut line = Vec::new();
        let mut position = 0;
        loop {
            match read_line_bounded(&mut reader, &mut line, limit).expect("in-memory read") {
                LineRead::End => break,
                LineRead::Complete(length) => {
                    let length = usize::try_from(length).expect("line length");
                    assert!(length <= limit);
                    assert_eq!(line, input[position..position + length]);
                    assert_eq!(line.last(), Some(&b'\n'));
                    position += length;
                }
                LineRead::Oversized(length) => {
                    let length = usize::try_from(length).expect("line length");
                    assert!(length > limit);
                    assert_eq!(input[position + length - 1], b'\n');
                    position += length;
                }
            }
        }
        let complete = input
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        assert_eq!(position, complete);
    }
}
