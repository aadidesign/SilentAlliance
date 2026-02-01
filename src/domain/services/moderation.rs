//! Content moderation service
//!
//! Handles content filtering, spam detection, and moderation actions.

use std::collections::HashSet;
use once_cell::sync::Lazy;
use regex::Regex;
use ammonia::Builder;

/// Content moderation service
pub struct ModerationService;

impl ModerationService {
    /// Sanitize HTML content (strip dangerous tags, allow safe formatting)
    pub fn sanitize_html(content: &str) -> String {
        Builder::default()
            .tags(HashSet::from([
                "p", "br", "b", "i", "em", "strong", "a", "ul", "ol", "li",
                "blockquote", "code", "pre", "h1", "h2", "h3", "h4", "h5", "h6",
                "hr", "table", "thead", "tbody", "tr", "th", "td", "sup", "sub",
                "del", "s", "span",
            ]))
            .url_schemes(HashSet::from(["http", "https", "mailto"]))
            .link_rel(Some("noopener noreferrer nofollow"))
            .clean(content)
            .to_string()
    }

    /// Parse markdown to HTML and sanitize
    pub fn render_markdown(content: &str) -> String {
        use pulldown_cmark::{Parser, Options, html};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);

        let parser = Parser::new_ext(content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        Self::sanitize_html(&html_output)
    }

    /// Check content for spam indicators
    pub fn check_spam(content: &str) -> SpamCheckResult {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Check for excessive caps
        let caps_ratio = Self::caps_ratio(content);
        if caps_ratio > 0.5 {
            score += 0.3;
            reasons.push("Excessive capitalization".to_string());
        }

        // Check for repeated characters
        if REPEATED_CHARS_REGEX.is_match(content) {
            score += 0.2;
            reasons.push("Repeated characters".to_string());
        }

        // Check for URL spam
        let url_count = URL_REGEX.find_iter(content).count();
        if url_count > 3 {
            score += 0.2 * (url_count as f32 - 3.0).min(2.0);
            reasons.push("Multiple URLs".to_string());
        }

        // Check for common spam patterns
        let lower_content = content.to_lowercase();
        for pattern in SPAM_PATTERNS.iter() {
            if lower_content.contains(pattern) {
                score += 0.4;
                reasons.push(format!("Spam pattern: {}", pattern));
            }
        }

        // Check for excessive emoji
        let emoji_count = content.chars().filter(|c| is_emoji(*c)).count();
        if emoji_count > 10 {
            score += 0.15;
            reasons.push("Excessive emoji".to_string());
        }

        SpamCheckResult {
            is_spam: score >= 0.5,
            score: score.min(1.0),
            reasons,
        }
    }

    /// Calculate ratio of uppercase letters
    fn caps_ratio(content: &str) -> f32 {
        let letters: Vec<char> = content.chars().filter(|c| c.is_alphabetic()).collect();
        if letters.is_empty() {
            return 0.0;
        }
        let uppercase = letters.iter().filter(|c| c.is_uppercase()).count();
        uppercase as f32 / letters.len() as f32
    }

    /// Check if content contains prohibited words
    pub fn check_prohibited_content(content: &str) -> ProhibitedContentResult {
        let lower_content = content.to_lowercase();
        let mut violations = Vec::new();

        // Check for prohibited words
        for word in PROHIBITED_WORDS.iter() {
            if lower_content.contains(word) {
                violations.push(ContentViolation {
                    violation_type: ViolationType::ProhibitedWord,
                    description: "Content contains prohibited terms".to_string(),
                    severity: Severity::High,
                });
            }
        }

        // Check for personal information patterns
        if EMAIL_REGEX.is_match(content) {
            violations.push(ContentViolation {
                violation_type: ViolationType::PersonalInfo,
                description: "Content appears to contain email addresses".to_string(),
                severity: Severity::Medium,
            });
        }

        if PHONE_REGEX.is_match(content) {
            violations.push(ContentViolation {
                violation_type: ViolationType::PersonalInfo,
                description: "Content appears to contain phone numbers".to_string(),
                severity: Severity::Medium,
            });
        }

        ProhibitedContentResult {
            has_violations: !violations.is_empty(),
            violations,
        }
    }

    /// Check content for rate limiting purposes
    pub fn should_rate_limit(
        recent_posts_count: i32,
        karma: i32,
        account_age_hours: i64,
    ) -> RateLimitDecision {
        // New accounts with low karma get stricter limits
        let is_new_account = account_age_hours < 24;
        let is_low_karma = karma < 10;

        let (limit, window_minutes) = match (is_new_account, is_low_karma) {
            (true, true) => (3, 60),   // 3 posts per hour
            (true, false) => (10, 60), // 10 posts per hour
            (false, true) => (5, 60),  // 5 posts per hour
            (false, false) => (30, 60), // 30 posts per hour (basically no limit)
        };

        if recent_posts_count >= limit {
            RateLimitDecision::Limited {
                retry_after_minutes: window_minutes / 2,
                reason: if is_new_account {
                    "New accounts have posting limits".to_string()
                } else {
                    "Posting too frequently".to_string()
                },
            }
        } else {
            RateLimitDecision::Allowed {
                remaining: limit - recent_posts_count,
            }
        }
    }

    /// Generate a moderation action
    pub fn create_action(
        action_type: ModerationType,
        target_id: uuid::Uuid,
        moderator_id: uuid::Uuid,
        reason: &str,
    ) -> ModerationAction {
        ModerationAction {
            id: uuid::Uuid::new_v4(),
            action_type,
            target_id,
            moderator_id,
            reason: reason.to_string(),
            created_at: chrono::Utc::now(),
        }
    }
}

/// Spam check result
#[derive(Debug, Clone)]
pub struct SpamCheckResult {
    pub is_spam: bool,
    pub score: f32,
    pub reasons: Vec<String>,
}

/// Prohibited content check result
#[derive(Debug, Clone)]
pub struct ProhibitedContentResult {
    pub has_violations: bool,
    pub violations: Vec<ContentViolation>,
}

/// Content violation
#[derive(Debug, Clone)]
pub struct ContentViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: Severity,
}

/// Violation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationType {
    ProhibitedWord,
    PersonalInfo,
    Spam,
    Harassment,
    IllegalContent,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Rate limit decision
#[derive(Debug, Clone)]
pub enum RateLimitDecision {
    Allowed { remaining: i32 },
    Limited { retry_after_minutes: i32, reason: String },
}

/// Moderation action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModerationType {
    RemovePost,
    RemoveComment,
    SuspendUser,
    WarnUser,
    BanFromSpace,
    Mute,
    ApproveContent,
    LockPost,
    PinPost,
}

/// Moderation action record
#[derive(Debug, Clone)]
pub struct ModerationAction {
    pub id: uuid::Uuid,
    pub action_type: ModerationType,
    pub target_id: uuid::Uuid,
    pub moderator_id: uuid::Uuid,
    pub reason: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Regex patterns
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https?://[^\s]+").unwrap()
});

static REPEATED_CHARS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(.)\1{4,}").unwrap()
});

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(\+?1?[-.\s]?)?\(?[0-9]{3}\)?[-.\s]?[0-9]{3}[-.\s]?[0-9]{4}\b").unwrap()
});

// Spam patterns (simplified for example)
static SPAM_PATTERNS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "free money",
        "click here now",
        "limited time offer",
        "act now",
        "100% free",
        "buy now",
        "earn money fast",
        "work from home",
        "congratulations you won",
        "claim your prize",
    ]
});

// Prohibited words (very simplified - real implementation would be more comprehensive)
static PROHIBITED_WORDS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        // This would contain actual prohibited content
        // Keeping minimal for example
    ]
});

/// Check if a character is an emoji
fn is_emoji(c: char) -> bool {
    // Simplified emoji detection
    matches!(c as u32,
        0x1F300..=0x1F9FF | // Various symbols and pictographs
        0x2600..=0x26FF |   // Misc symbols
        0x2700..=0x27BF |   // Dingbats
        0x1F600..=0x1F64F | // Emoticons
        0x1F680..=0x1F6FF   // Transport and map symbols
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_sanitization() {
        let dangerous = r#"<script>alert('xss')</script><p>Hello</p>"#;
        let safe = ModerationService::sanitize_html(dangerous);
        assert!(!safe.contains("<script>"));
        assert!(safe.contains("<p>"));
    }

    #[test]
    fn test_spam_detection() {
        let spam = "FREE MONEY!!! Click here now!!! CLICK CLICK CLICK aaaaaaaa";
        let result = ModerationService::check_spam(spam);
        assert!(result.is_spam);

        let normal = "This is a normal message about programming.";
        let result = ModerationService::check_spam(normal);
        assert!(!result.is_spam);
    }

    #[test]
    fn test_rate_limit_decision() {
        // New account, low karma
        let decision = ModerationService::should_rate_limit(5, 0, 12);
        assert!(matches!(decision, RateLimitDecision::Limited { .. }));

        // Established account
        let decision = ModerationService::should_rate_limit(5, 100, 1000);
        assert!(matches!(decision, RateLimitDecision::Allowed { .. }));
    }

    #[test]
    fn test_markdown_rendering() {
        let md = "# Hello\n\n**bold** and *italic*\n\n- list item";
        let html = ModerationService::render_markdown(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
        assert!(html.contains("<em>"));
    }
}
