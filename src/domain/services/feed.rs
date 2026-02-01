//! Feed and ranking algorithms
//!
//! Implements Reddit-style ranking algorithms for posts and comments.

use chrono::{DateTime, Duration, Utc};

/// Calculate hot score for a post (Reddit's hot ranking algorithm)
///
/// This algorithm factors in:
/// - Net score (upvotes - downvotes)
/// - Time since posting (decay)
///
/// Formula based on Reddit's original hot ranking:
/// hot = log10(max(|score|, 1)) * sign(score) + (timestamp / 45000)
pub fn calculate_hot_score(upvotes: i32, downvotes: i32, created_at: DateTime<Utc>) -> f64 {
    let score = upvotes - downvotes;
    let order = (score.abs().max(1) as f64).log10();
    let sign = if score > 0 {
        1.0
    } else if score < 0 {
        -1.0
    } else {
        0.0
    };

    // Epoch: Jan 1, 2024
    let epoch = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let seconds = (created_at - epoch).num_seconds() as f64;

    // The divisor determines how quickly posts decay
    // 45000 = about 12.5 hours for decay
    sign * order + seconds / 45000.0
}

/// Calculate Wilson score for ranking (for confidence in small sample sizes)
///
/// This is useful for sorting when you want to account for
/// statistical confidence with limited votes.
pub fn calculate_wilson_score(upvotes: i32, downvotes: i32) -> f64 {
    let n = (upvotes + downvotes) as f64;
    if n == 0.0 {
        return 0.0;
    }

    let z = 1.96; // 95% confidence
    let p = upvotes as f64 / n;

    // Wilson score interval lower bound
    (p + z * z / (2.0 * n) - z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt())
        / (1.0 + z * z / n)
}

/// Calculate controversy score
///
/// Posts are controversial when they have many votes but the ratio is close to 50/50
pub fn calculate_controversy_score(upvotes: i32, downvotes: i32) -> f64 {
    let total = upvotes + downvotes;
    if total == 0 {
        return 0.0;
    }

    let magnitude = total as f64;
    let balance = if upvotes > downvotes {
        downvotes as f64 / upvotes as f64
    } else if downvotes > upvotes {
        upvotes as f64 / downvotes as f64
    } else {
        1.0
    };

    // Controversy = magnitude * balance
    // High controversy = many votes AND close to 50/50
    magnitude.powf(balance)
}

/// Calculate rising score
///
/// Factors in recent voting activity compared to age
pub fn calculate_rising_score(
    upvotes: i32,
    downvotes: i32,
    created_at: DateTime<Utc>,
    recent_upvotes: i32,
    recent_downvotes: i32,
) -> f64 {
    let age_hours = (Utc::now() - created_at).num_hours().max(1) as f64;
    let total_score = upvotes - downvotes;
    let recent_score = recent_upvotes - recent_downvotes;

    // Weight recent activity more heavily
    let recent_weight = 2.0;
    let weighted_score = total_score as f64 + (recent_score as f64 * recent_weight);

    // Normalize by age
    weighted_score / age_hours.sqrt()
}

/// Calculate "best" score for comments (Reddit's best algorithm)
///
/// Uses Wilson score for confidence-adjusted ranking
pub fn calculate_best_score(upvotes: i32, downvotes: i32) -> f64 {
    calculate_wilson_score(upvotes, downvotes)
}

/// Get the start timestamp for a time range filter
pub fn time_range_start(range: crate::domain::entities::TimeRange) -> DateTime<Utc> {
    let now = Utc::now();
    match range {
        crate::domain::entities::TimeRange::Hour => now - Duration::hours(1),
        crate::domain::entities::TimeRange::Day => now - Duration::days(1),
        crate::domain::entities::TimeRange::Week => now - Duration::weeks(1),
        crate::domain::entities::TimeRange::Month => now - Duration::days(30),
        crate::domain::entities::TimeRange::Year => now - Duration::days(365),
        crate::domain::entities::TimeRange::All => DateTime::UNIX_EPOCH.into(),
    }
}

/// Build SQL ORDER BY clause for post sorting
pub fn post_sort_order_by(sort: crate::domain::entities::PostSort) -> &'static str {
    match sort {
        crate::domain::entities::PostSort::Hot => "score DESC, created_at DESC",
        crate::domain::entities::PostSort::New => "created_at DESC",
        crate::domain::entities::PostSort::Top => "(upvotes - downvotes) DESC, created_at DESC",
        crate::domain::entities::PostSort::Rising => "score DESC, created_at DESC",
        crate::domain::entities::PostSort::Controversial => "score DESC, created_at DESC",
    }
}

/// Calculate and update scores for a batch of posts
pub struct ScoreCalculator;

impl ScoreCalculator {
    /// Calculate scores for a post
    pub fn calculate_post_scores(
        upvotes: i32,
        downvotes: i32,
        created_at: DateTime<Utc>,
    ) -> PostScores {
        PostScores {
            hot: calculate_hot_score(upvotes, downvotes, created_at),
            wilson: calculate_wilson_score(upvotes, downvotes),
            controversy: calculate_controversy_score(upvotes, downvotes),
        }
    }

    /// Calculate score for a comment
    pub fn calculate_comment_score(upvotes: i32, downvotes: i32) -> f64 {
        calculate_wilson_score(upvotes, downvotes)
    }
}

/// Calculated scores for a post
#[derive(Debug, Clone)]
pub struct PostScores {
    pub hot: f64,
    pub wilson: f64,
    pub controversy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_score_ordering() {
        let now = Utc::now();
        let one_hour_ago = now - Duration::hours(1);
        let one_day_ago = now - Duration::days(1);

        // Same votes, newer post should score higher
        let score_new = calculate_hot_score(100, 10, now);
        let score_old = calculate_hot_score(100, 10, one_day_ago);
        assert!(score_new > score_old);

        // Same age, more upvotes should score higher
        let score_high = calculate_hot_score(1000, 10, one_hour_ago);
        let score_low = calculate_hot_score(100, 10, one_hour_ago);
        assert!(score_high > score_low);

        // Negative scores should be lower than positive
        let score_pos = calculate_hot_score(100, 10, one_hour_ago);
        let score_neg = calculate_hot_score(10, 100, one_hour_ago);
        assert!(score_pos > score_neg);
    }

    #[test]
    fn test_wilson_score() {
        // 100% upvote rate with few votes should score lower than
        // 90% upvote rate with many votes (statistical confidence)
        let score_few = calculate_wilson_score(5, 0);
        let score_many = calculate_wilson_score(90, 10);
        assert!(score_many > score_few);

        // Zero votes should return 0
        let score_zero = calculate_wilson_score(0, 0);
        assert_eq!(score_zero, 0.0);
    }

    #[test]
    fn test_controversy_score() {
        // Even split should be more controversial than lopsided
        let score_even = calculate_controversy_score(100, 100);
        let score_lopsided = calculate_controversy_score(100, 10);
        assert!(score_even > score_lopsided);

        // More votes means more controversial (when balanced)
        let score_high = calculate_controversy_score(1000, 900);
        let score_low = calculate_controversy_score(10, 9);
        assert!(score_high > score_low);
    }
}
