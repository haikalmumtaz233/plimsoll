use std::iter::Sum;
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TokenCounts {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
}

impl TokenCounts {
    #[must_use]
    pub const fn total(self) -> u64 {
        self.input
            .saturating_add(self.output)
            .saturating_add(self.cache_creation)
            .saturating_add(self.cache_read)
    }
}

impl Add for TokenCounts {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            input: self.input.saturating_add(other.input),
            output: self.output.saturating_add(other.output),
            cache_creation: self.cache_creation.saturating_add(other.cache_creation),
            cache_read: self.cache_read.saturating_add(other.cache_read),
        }
    }
}

impl AddAssign for TokenCounts {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sum for TokenCounts {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::TokenCounts;

    const SAMPLE: TokenCounts = TokenCounts {
        input: 1,
        output: 20,
        cache_creation: 300,
        cache_read: 4_000,
    };

    #[test]
    fn total_sums_every_kind() {
        assert_eq!(SAMPLE.total(), 4_321);
        assert_eq!(TokenCounts::default().total(), 0);
    }

    #[test]
    fn addition_is_per_kind() {
        let doubled = SAMPLE + SAMPLE;
        assert_eq!(doubled.input, 2);
        assert_eq!(doubled.output, 40);
        assert_eq!(doubled.cache_creation, 600);
        assert_eq!(doubled.cache_read, 8_000);
    }

    #[test]
    fn add_assign_and_sum_agree_with_addition() {
        let mut accumulated = TokenCounts::default();
        accumulated += SAMPLE;
        accumulated += SAMPLE;
        let summed: TokenCounts = [SAMPLE, SAMPLE].into_iter().sum();
        assert_eq!(accumulated, SAMPLE + SAMPLE);
        assert_eq!(summed, accumulated);
    }

    #[test]
    fn arithmetic_saturates() {
        let huge = TokenCounts {
            input: u64::MAX,
            ..TokenCounts::default()
        };
        assert_eq!((huge + SAMPLE).input, u64::MAX);
        assert_eq!((huge + SAMPLE).total(), u64::MAX);
    }
}
