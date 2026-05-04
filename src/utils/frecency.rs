use super::config;
use super::types::{Frecency, NotedCount, UnixTimestampHours};

fn calculate_with_lambda(
    count: NotedCount,
    last_noted_timestamp_hours: UnixTimestampHours,
    highest_count: NotedCount,
    oldest_last_noted_timestamp_hours: UnixTimestampHours,
    lambda: f64,
) -> Frecency {
    let freq_score = if highest_count > 0 {
        count as f64 / highest_count as f64
    } else {
        0.0
    };

    let recency_score = if last_noted_timestamp_hours < oldest_last_noted_timestamp_hours {
        1.0 - (last_noted_timestamp_hours / oldest_last_noted_timestamp_hours)
    } else {
        0.0
    };

    (1.0 - lambda).mul_add(freq_score, lambda * recency_score)
}

pub fn calculate(
    count: NotedCount,
    last_noted_timestamp_hours: UnixTimestampHours,
    highest_count: NotedCount,
    oldest_last_noted_timestamp_hours: UnixTimestampHours,
) -> Frecency {
    let lambda = config::get_recency_bias();
    calculate_with_lambda(
        count,
        last_noted_timestamp_hours,
        highest_count,
        oldest_last_noted_timestamp_hours,
        lambda,
    )
}

#[allow(clippy::unwrap_used, reason = "unwrap() OK inside tests")]
#[allow(
    clippy::float_cmp,
    reason = "Exact float comparisons intentional in tests"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_zero_highest_count_zeros_freq_component() {
        // When highest_count = 0, freq_score = 0; with lambda=0, full result is 0
        assert!((calculate_with_lambda(5, 10.0, 0, 100.0, 0.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pure_frequency_half() {
        // lambda=0: pure frequency, count=5, highest=10 → 0.5
        let result = calculate_with_lambda(5, 10.0, 10, 100.0, 0.0);
        assert!((result - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pure_frequency_full() {
        // lambda=0: count == highest_count → 1.0
        let result = calculate_with_lambda(10, 10.0, 10, 100.0, 0.0);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_recency_when_not_older_than_oldest() {
        // last_noted >= oldest → recency_score = 0; with lambda=1, result = 0
        assert!((calculate_with_lambda(10, 100.0, 10, 50.0, 1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_max_recency_when_just_noted() {
        // last_noted=0 < oldest → recency_score = 1.0; with lambda=1, result = 1.0
        let result = calculate_with_lambda(10, 0.0, 10, 100.0, 1.0);
        assert!((result - 1.0).abs() < f64::EPSILON);
    }

    proptest! {
        #[test]
        fn prop_frecency_always_in_unit_range(
            count in 0u64..=1000u64,
            extra in 0u64..=1000u64,
            last_noted_hours in 0.0f64..999.0f64,
            oldest_hours in 1.0f64..1000.0f64,
            lambda in 0.0f64..=1.0f64,
        ) {
            let highest_count = count + extra;
            let result = calculate_with_lambda(count, last_noted_hours, highest_count, oldest_hours, lambda);
            prop_assert!(result >= 0.0, "frecency {result} should be >= 0");
            prop_assert!(result <= 1.0, "frecency {result} should be <= 1");
        }

        #[test]
        fn prop_higher_count_gives_higher_frecency(
            count1 in 0u64..=499u64,
            diff in 1u64..=500u64,
            last_noted_hours in 0.0f64..50.0f64,
            oldest_hours in 100.0f64..1000.0f64,
        ) {
            // With lambda=0 (pure frequency), strictly higher count → strictly higher frecency
            let count2 = count1 + diff;
            let highest = count2 + 1;
            let result1 = calculate_with_lambda(count1, last_noted_hours, highest, oldest_hours, 0.0);
            let result2 = calculate_with_lambda(count2, last_noted_hours, highest, oldest_hours, 0.0);
            prop_assert!(result2 > result1,
                "count2={count2} should score higher than count1={count1}");
        }

        #[test]
        fn prop_more_recent_gives_higher_frecency(
            count in 1u64..=1000u64,
            hours_recent in 0.0f64..49.0f64,
            hours_older in 50.0f64..99.0f64,
            oldest_hours in 100.0f64..1000.0f64,
        ) {
            // With lambda=1 (pure recency), lower age-hours (more recent) → higher frecency
            let result_recent = calculate_with_lambda(count, hours_recent, count, oldest_hours, 1.0);
            let result_older  = calculate_with_lambda(count, hours_older,  count, oldest_hours, 1.0);
            prop_assert!(result_recent > result_older,
                "hours_recent={hours_recent} should score higher than hours_older={hours_older}");
        }
    }
}
