//! Domain entities for SilentAlliance
//!
//! These types represent the core business objects in the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ==================== Identity ====================

/// Pseudonymous user identity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Identity {
    pub id: Uuid,
    #[serde(skip_serializing)]
    pub public_key: Vec<u8>,
    pub public_key_fingerprint: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
    pub karma: i32,
    pub is_verified: bool,
    pub is_suspended: bool,
    pub suspended_reason: Option<String>,
    pub suspended_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public view of an identity (safe to expose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityPublic {
    pub id: Uuid,
    pub public_key_fingerprint: String,
    pub display_name: Option<String>,
    pub avatar_hash: Option<String>,
    pub bio: Option<String>,
    pub karma: i32,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<Identity> for IdentityPublic {
    fn from(identity: Identity) -> Self {
        Self {
            id: identity.id,
            public_key_fingerprint: identity.public_key_fingerprint,
            display_name: identity.display_name,
            avatar_hash: identity.avatar_hash,
            bio: identity.bio,
            karma: identity.karma,
            is_verified: identity.is_verified,
            created_at: identity.created_at,
        }
    }
}

/// Create identity request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateIdentityRequest {
    #[validate(length(min = 32, max = 64, message = "Invalid public key length"))]
    pub public_key: String, // Base64 encoded
    #[validate(length(min = 1, max = 50, message = "Display name must be 1-50 characters"))]
    pub display_name: Option<String>,
}

/// Update identity request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateIdentityRequest {
    #[validate(length(min = 1, max = 50, message = "Display name must be 1-50 characters"))]
    pub display_name: Option<String>,
    #[validate(length(max = 500, message = "Bio must be at most 500 characters"))]
    pub bio: Option<String>,
}

// ==================== Credentials ====================

/// User credentials for authentication
#[derive(Debug, Clone, FromRow)]
pub struct Credential {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub credential_type: String,
    pub credential_hash: Option<String>,
    pub oauth_provider: Option<String>,
    pub oauth_subject: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ==================== Refresh Tokens ====================

/// Refresh token for JWT refresh
#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub token_hash: String,
    pub family_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

// ==================== Spaces ====================

/// Community/Space (like subreddits)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Space {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub rules: serde_json::Value,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub is_private: bool,
    pub is_nsfw: bool,
    pub creator_id: Option<Uuid>,
    pub subscriber_count: i32,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create space request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateSpaceRequest {
    #[validate(length(min = 3, max = 50, message = "Name must be 3-50 characters"))]
    #[validate(regex(path = "crate::domain::entities::SLUG_REGEX", message = "Name can only contain letters, numbers, and underscores"))]
    pub name: String,
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub is_nsfw: Option<bool>,
}

/// Update space request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateSpaceRequest {
    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
    pub rules: Option<Vec<String>>,
    pub is_private: Option<bool>,
    pub is_nsfw: Option<bool>,
}

/// Space membership
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpaceMember {
    pub id: Uuid,
    pub space_id: Uuid,
    pub identity_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

/// Member roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Member,
    Moderator,
    Admin,
}

impl Default for MemberRole {
    fn default() -> Self {
        Self::Member
    }
}

// ==================== Posts ====================

/// Post in a space
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: Uuid,
    pub space_id: Uuid,
    pub author_id: Option<Uuid>,
    pub title: String,
    pub content: Option<String>,
    pub content_type: ContentType,
    pub url: Option<String>,
    pub media_ids: Vec<Uuid>,
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i32,
    pub comment_count: i32,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub is_removed: bool,
    pub removed_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Post with additional context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostWithContext {
    #[serde(flatten)]
    pub post: Post,
    pub author: Option<IdentityPublic>,
    pub space: Option<SpaceSummary>,
    pub user_vote: Option<i16>,
}

/// Content types for posts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Link,
    Media,
    Poll,
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Text
    }
}

/// Create post request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePostRequest {
    #[validate(length(min = 1, max = 300, message = "Title must be 1-300 characters"))]
    pub title: String,
    #[validate(length(max = 40000, message = "Content must be at most 40000 characters"))]
    pub content: Option<String>,
    pub content_type: Option<ContentType>,
    #[validate(url(message = "Invalid URL"))]
    pub url: Option<String>,
    pub media_ids: Option<Vec<Uuid>>,
}

/// Update post request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePostRequest {
    #[validate(length(max = 40000, message = "Content must be at most 40000 characters"))]
    pub content: Option<String>,
}

// ==================== Comments ====================

/// Comment on a post
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub content: String,
    pub depth: i32,
    pub path: String,
    pub upvotes: i32,
    pub downvotes: i32,
    pub score: i32,
    pub is_removed: bool,
    pub removed_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithContext {
    #[serde(flatten)]
    pub comment: Comment,
    pub author: Option<IdentityPublic>,
    pub user_vote: Option<i16>,
    pub replies: Vec<CommentWithContext>,
}

/// Create comment request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCommentRequest {
    pub parent_id: Option<Uuid>,
    #[validate(length(min = 1, max = 10000, message = "Content must be 1-10000 characters"))]
    pub content: String,
}

/// Update comment request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, max = 10000, message = "Content must be 1-10000 characters"))]
    pub content: String,
}

// ==================== Votes ====================

/// Vote on a post or comment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub target_type: VoteTargetType,
    pub target_id: Uuid,
    pub vote_value: i16, // 1 for upvote, -1 for downvote
    pub created_at: DateTime<Utc>,
}

/// Vote target types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VoteTargetType {
    Post,
    Comment,
}

/// Vote request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VoteRequest {
    #[validate(range(min = -1, max = 1, message = "Vote value must be -1, 0, or 1"))]
    pub value: i16,
}

// ==================== Messages ====================

/// Conversation for direct messages
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conversation participant
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationParticipant {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub identity_id: Uuid,
    pub encrypted_key: Vec<u8>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Encrypted message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub encrypted_content: Vec<u8>,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Message with decryption info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub sender: Option<IdentityPublic>,
    pub encrypted_content: String, // Base64
    pub nonce: String,             // Base64
    pub created_at: DateTime<Utc>,
}

/// Create conversation request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateConversationRequest {
    #[validate(length(min = 1, message = "At least one participant required"))]
    pub participant_ids: Vec<Uuid>,
    pub initial_message: Option<EncryptedMessageRequest>,
}

/// Encrypted message request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessageRequest {
    pub encrypted_content: String, // Base64
    pub nonce: String,             // Base64
}

// ==================== Media ====================

/// Stored media file
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Media {
    pub id: Uuid,
    pub uploader_id: Option<Uuid>,
    pub file_hash: String,
    pub mime_type: String,
    pub file_size: i32,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub is_processed: bool,
    pub created_at: DateTime<Utc>,
}

// ==================== Reports ====================

/// Content report
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: Uuid,
    pub reporter_id: Option<Uuid>,
    pub target_type: ReportTargetType,
    pub target_id: Uuid,
    pub reason: ReportReason,
    pub description: Option<String>,
    pub status: ReportStatus,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

/// Report target types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ReportTargetType {
    Post,
    Comment,
    Message,
    Identity,
    Space,
}

/// Report reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ReportReason {
    Spam,
    Harassment,
    HateSpeech,
    Violence,
    Misinformation,
    IllegalContent,
    PrivacyViolation,
    Impersonation,
    Other,
}

/// Report statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    Pending,
    Reviewed,
    Actioned,
    Dismissed,
}

impl Default for ReportStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Create report request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateReportRequest {
    pub target_type: ReportTargetType,
    pub target_id: Uuid,
    pub reason: ReportReason,
    #[validate(length(max = 1000, message = "Description must be at most 1000 characters"))]
    pub description: Option<String>,
}

// ==================== Notifications ====================

/// Notification
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub notification_type: NotificationType,
    pub payload: serde_json::Value,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    PostReply,
    CommentReply,
    Mention,
    NewMessage,
    SpaceInvite,
    ModeratorAction,
    SystemAlert,
}

// ==================== Pagination ====================

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[validate(range(min = 0))]
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    25
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: 25,
            offset: 0,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

impl PaginationInfo {
    pub fn new(total: i64, limit: i64, offset: i64) -> Self {
        Self {
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        }
    }
}

// ==================== Feed & Sorting ====================

/// Sort options for posts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PostSort {
    #[default]
    Hot,
    New,
    Top,
    Rising,
    Controversial,
}

/// Time range for sorting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    Hour,
    Day,
    Week,
    Month,
    Year,
    #[default]
    All,
}

/// Feed query parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FeedParams {
    #[serde(default)]
    pub sort: PostSort,
    #[serde(default)]
    pub time_range: TimeRange,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

// ==================== Summary Types ====================

/// Space summary for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceSummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub icon_url: Option<String>,
    pub subscriber_count: i32,
}

impl From<Space> for SpaceSummary {
    fn from(space: Space) -> Self {
        Self {
            id: space.id,
            name: space.name,
            slug: space.slug,
            icon_url: space.icon_url,
            subscriber_count: space.subscriber_count,
        }
    }
}

// ==================== Regex Patterns ====================

use once_cell::sync::Lazy;
use regex::Regex;

pub static SLUG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9_]+$").unwrap()
});
