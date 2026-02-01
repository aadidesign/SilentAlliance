//! Karma calculation service
//!
//! Handles karma points calculation for user activities.

use chrono::{DateTime, Duration, Utc};

/// Karma calculation service
pub struct KarmaService;

impl KarmaService {
    /// Calculate karma delta for a vote change
    ///
    /// Returns (post_author_karma_delta, voter_karma_delta)
    pub fn calculate_vote_karma(
        old_vote: Option<i16>,
        new_vote: i16,
        content_age: Duration,
    ) -> (i32, i32) {
        // Karma for receiving votes decreases as content ages
        let age_multiplier = Self::age_decay_multiplier(content_age);

        let old_value = old_vote.unwrap_or(0);
        let vote_delta = new_vote - old_value;

        // Post author karma change (scaled by age)
        let author_karma = (vote_delta as f64 * age_multiplier).round() as i32;

        // Voters don't gain/lose karma for voting
        let voter_karma = 0;

        (author_karma, voter_karma)
    }

    /// Calculate karma for creating a post
    pub fn post_creation_karma() -> i32 {
        1 // Small karma boost for creating content
    }

    /// Calculate karma for creating a comment
    pub fn comment_creation_karma() -> i32 {
        1
    }

    /// Calculate karma penalty for content removal
    pub fn removal_karma_penalty(upvotes: i32, downvotes: i32) -> i32 {
        // Lose karma equal to net score (capped)
        let net_score = upvotes - downvotes;
        -net_score.max(0).min(100) // Lose at most 100 karma
    }

    /// Calculate karma decay multiplier based on content age
    fn age_decay_multiplier(age: Duration) -> f64 {
        let hours = age.num_hours().max(0) as f64;

        // Full karma for first 24 hours
        // 50% karma for 24-72 hours
        // 25% karma for 72+ hours
        if hours < 24.0 {
            1.0
        } else if hours < 72.0 {
            0.5
        } else if hours < 168.0 {
            // 1 week
            0.25
        } else {
            0.1
        }
    }

    /// Check if user has enough karma for an action
    pub fn check_karma_requirement(
        current_karma: i32,
        required_karma: i32,
        action: &str,
    ) -> KarmaCheck {
        if current_karma >= required_karma {
            KarmaCheck::Allowed
        } else {
            KarmaCheck::InsufficientKarma {
                current: current_karma,
                required: required_karma,
                action: action.to_string(),
            }
        }
    }

    /// Get karma thresholds for various actions
    pub fn get_thresholds() -> KarmaThresholds {
        KarmaThresholds {
            create_space: 100,
            create_post: 0,
            create_comment: 0,
            downvote: 10,
            send_dm: 10,
            upload_media: 25,
            no_rate_limit: 1000,
        }
    }

    /// Calculate karma level/tier
    pub fn calculate_level(karma: i32) -> KarmaLevel {
        match karma {
            k if k < 0 => KarmaLevel::Restricted,
            k if k < 10 => KarmaLevel::Newcomer,
            k if k < 100 => KarmaLevel::Regular,
            k if k < 500 => KarmaLevel::Trusted,
            k if k < 1000 => KarmaLevel::Established,
            k if k < 5000 => KarmaLevel::Veteran,
            _ => KarmaLevel::Legend,
        }
    }
}

/// Result of karma check
#[derive(Debug, Clone)]
pub enum KarmaCheck {
    Allowed,
    InsufficientKarma {
        current: i32,
        required: i32,
        action: String,
    },
}

impl KarmaCheck {
    pub fn is_allowed(&self) -> bool {
        matches!(self, KarmaCheck::Allowed)
    }
}

/// Karma thresholds for actions
#[derive(Debug, Clone)]
pub struct KarmaThresholds {
    pub create_space: i32,
    pub create_post: i32,
    pub create_comment: i32,
    pub downvote: i32,
    pub send_dm: i32,
    pub upload_media: i32,
    pub no_rate_limit: i32,
}

/// Karma levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KarmaLevel {
    Restricted,
    Newcomer,
    Regular,
    Trusted,
    Established,
    Veteran,
    Legend,
}

impl KarmaLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Restricted => "Restricted",
            Self::Newcomer => "Newcomer",
            Self::Regular => "Regular",
            Self::Trusted => "Trusted",
            Self::Established => "Established",
            Self::Veteran => "Veteran",
            Self::Legend => "Legend",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Restricted => "Limited actions due to negative karma",
            Self::Newcomer => "New to the community",
            Self::Regular => "Active community member",
            Self::Trusted => "Trusted contributor",
            Self::Established => "Well-established presence",
            Self::Veteran => "Long-time valued member",
            Self::Legend => "Community legend",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_karma_calculation() {
        // Fresh content
        let (author_karma, _) = KarmaService::calculate_vote_karma(None, 1, Duration::zero());
        assert_eq!(author_karma, 1);

        // Changing vote from up to down
        let (author_karma, _) = KarmaService::calculate_vote_karma(Some(1), -1, Duration::zero());
        assert_eq!(author_karma, -2);

        // Old content (reduced karma)
        let (author_karma, _) =
            KarmaService::calculate_vote_karma(None, 1, Duration::days(7));
        assert!(author_karma < 1);
    }

    #[test]
    fn test_karma_levels() {
        assert_eq!(KarmaService::calculate_level(-10), KarmaLevel::Restricted);
        assert_eq!(KarmaService::calculate_level(0), KarmaLevel::Newcomer);
        assert_eq!(KarmaService::calculate_level(50), KarmaLevel::Regular);
        assert_eq!(KarmaService::calculate_level(200), KarmaLevel::Trusted);
        assert_eq!(KarmaService::calculate_level(750), KarmaLevel::Established);
        assert_eq!(KarmaService::calculate_level(2000), KarmaLevel::Veteran);
        assert_eq!(KarmaService::calculate_level(10000), KarmaLevel::Legend);
    }

    #[test]
    fn test_karma_check() {
        let check = KarmaService::check_karma_requirement(50, 100, "create_space");
        assert!(!check.is_allowed());

        let check = KarmaService::check_karma_requirement(150, 100, "create_space");
        assert!(check.is_allowed());
    }
}
