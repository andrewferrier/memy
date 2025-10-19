use crate::config;
use crate::types::{Frecency, NotedCount, UnixTimestampHours};

pub fn calculate(
    count: NotedCount,
    last_noted_timestamp_hours: UnixTimestampHours,
    highest_count: NotedCount,
    oldest_last_noted_timestamp_hours: UnixTimestampHours,
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

    let lambda = config::get_recency_bias();
    (1.0 - lambda).mul_add(freq_score, lambda * recency_score)
}
