//! Repository traits for data access
//!
//! These traits define the interface for data access operations,
//! allowing for different implementations (e.g., PostgreSQL, mock).

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::*;
use crate::errors::ApiResult;

// ==================== Identity Repository ====================

#[async_trait]
pub trait IdentityRepository: Send + Sync {
    /// Create a new identity
    async fn create(&self, public_key: &[u8], display_name: Option<&str>) -> ApiResult<Identity>;

    /// Find identity by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Identity>>;

    /// Find identity by public key fingerprint
    async fn find_by_fingerprint(&self, fingerprint: &str) -> ApiResult<Option<Identity>>;

    /// Find identity by public key
    async fn find_by_public_key(&self, public_key: &[u8]) -> ApiResult<Option<Identity>>;

    /// Update identity
    async fn update(&self, id: Uuid, display_name: Option<&str>, bio: Option<&str>) -> ApiResult<Identity>;

    /// Update karma
    async fn update_karma(&self, id: Uuid, delta: i32) -> ApiResult<()>;

    /// Suspend identity
    async fn suspend(&self, id: Uuid, reason: &str, until: Option<chrono::DateTime<chrono::Utc>>) -> ApiResult<()>;

    /// Unsuspend identity
    async fn unsuspend(&self, id: Uuid) -> ApiResult<()>;
}

// ==================== Credential Repository ====================

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    /// Create password credential
    async fn create_password(&self, identity_id: Uuid, password_hash: &str) -> ApiResult<Credential>;

    /// Create OAuth credential
    async fn create_oauth(
        &self,
        identity_id: Uuid,
        provider: &str,
        subject: &str,
    ) -> ApiResult<Credential>;

    /// Find by identity ID and type
    async fn find_by_identity_and_type(
        &self,
        identity_id: Uuid,
        credential_type: &str,
    ) -> ApiResult<Option<Credential>>;

    /// Find by OAuth provider and subject
    async fn find_by_oauth(&self, provider: &str, subject: &str) -> ApiResult<Option<Credential>>;

    /// Update password hash
    async fn update_password(&self, identity_id: Uuid, password_hash: &str) -> ApiResult<()>;

    /// Delete credential
    async fn delete(&self, id: Uuid) -> ApiResult<()>;
}

// ==================== Refresh Token Repository ====================

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    /// Create a new refresh token
    async fn create(
        &self,
        identity_id: Uuid,
        token_hash: &str,
        family_id: Uuid,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> ApiResult<RefreshToken>;

    /// Find by token hash
    async fn find_by_hash(&self, token_hash: &str) -> ApiResult<Option<RefreshToken>>;

    /// Revoke a token
    async fn revoke(&self, id: Uuid) -> ApiResult<()>;

    /// Revoke all tokens in a family (for token reuse detection)
    async fn revoke_family(&self, family_id: Uuid) -> ApiResult<()>;

    /// Revoke all tokens for an identity
    async fn revoke_all_for_identity(&self, identity_id: Uuid) -> ApiResult<()>;

    /// Delete expired tokens
    async fn delete_expired(&self) -> ApiResult<u64>;
}

// ==================== Space Repository ====================

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    /// Create a new space
    async fn create(&self, creator_id: Uuid, request: &CreateSpaceRequest) -> ApiResult<Space>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Space>>;

    /// Find by slug
    async fn find_by_slug(&self, slug: &str) -> ApiResult<Option<Space>>;

    /// Update space
    async fn update(&self, id: Uuid, request: &UpdateSpaceRequest) -> ApiResult<Space>;

    /// Delete space
    async fn delete(&self, id: Uuid) -> ApiResult<()>;

    /// List spaces with pagination
    async fn list(&self, pagination: &PaginationParams, search: Option<&str>) -> ApiResult<(Vec<Space>, i64)>;

    /// List spaces by member
    async fn list_by_member(&self, identity_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Space>, i64)>;

    /// Increment subscriber count
    async fn increment_subscriber_count(&self, id: Uuid, delta: i32) -> ApiResult<()>;

    /// Increment post count
    async fn increment_post_count(&self, id: Uuid, delta: i32) -> ApiResult<()>;
}

// ==================== Space Member Repository ====================

#[async_trait]
pub trait SpaceMemberRepository: Send + Sync {
    /// Add member to space
    async fn add(&self, space_id: Uuid, identity_id: Uuid, role: MemberRole) -> ApiResult<SpaceMember>;

    /// Remove member from space
    async fn remove(&self, space_id: Uuid, identity_id: Uuid) -> ApiResult<()>;

    /// Find membership
    async fn find(&self, space_id: Uuid, identity_id: Uuid) -> ApiResult<Option<SpaceMember>>;

    /// Update member role
    async fn update_role(&self, space_id: Uuid, identity_id: Uuid, role: MemberRole) -> ApiResult<()>;

    /// List members of a space
    async fn list_by_space(&self, space_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<SpaceMember>, i64)>;

    /// Check if user is member
    async fn is_member(&self, space_id: Uuid, identity_id: Uuid) -> ApiResult<bool>;

    /// Check if user is moderator or admin
    async fn is_moderator(&self, space_id: Uuid, identity_id: Uuid) -> ApiResult<bool>;
}

// ==================== Post Repository ====================

#[async_trait]
pub trait PostRepository: Send + Sync {
    /// Create a new post
    async fn create(&self, space_id: Uuid, author_id: Uuid, request: &CreatePostRequest) -> ApiResult<Post>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Post>>;

    /// Update post
    async fn update(&self, id: Uuid, request: &UpdatePostRequest) -> ApiResult<Post>;

    /// Delete post (soft delete)
    async fn delete(&self, id: Uuid, reason: Option<&str>) -> ApiResult<()>;

    /// List posts in a space
    async fn list_by_space(
        &self,
        space_id: Uuid,
        sort: PostSort,
        time_range: TimeRange,
        pagination: &PaginationParams,
    ) -> ApiResult<(Vec<Post>, i64)>;

    /// List posts by author
    async fn list_by_author(&self, author_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Post>, i64)>;

    /// Get feed (posts from multiple spaces)
    async fn get_feed(
        &self,
        space_ids: &[Uuid],
        sort: PostSort,
        time_range: TimeRange,
        pagination: &PaginationParams,
    ) -> ApiResult<(Vec<Post>, i64)>;

    /// Update vote counts
    async fn update_votes(&self, id: Uuid, upvotes: i32, downvotes: i32) -> ApiResult<()>;

    /// Update comment count
    async fn increment_comment_count(&self, id: Uuid, delta: i32) -> ApiResult<()>;

    /// Pin/unpin post
    async fn set_pinned(&self, id: Uuid, pinned: bool) -> ApiResult<()>;

    /// Lock/unlock post
    async fn set_locked(&self, id: Uuid, locked: bool) -> ApiResult<()>;
}

// ==================== Comment Repository ====================

#[async_trait]
pub trait CommentRepository: Send + Sync {
    /// Create a comment
    async fn create(&self, post_id: Uuid, author_id: Uuid, request: &CreateCommentRequest) -> ApiResult<Comment>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Comment>>;

    /// Update comment
    async fn update(&self, id: Uuid, content: &str) -> ApiResult<Comment>;

    /// Delete comment (soft delete)
    async fn delete(&self, id: Uuid, reason: Option<&str>) -> ApiResult<()>;

    /// List comments for a post (returns flat list, client builds tree)
    async fn list_by_post(&self, post_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Comment>, i64)>;

    /// List comments by author
    async fn list_by_author(&self, author_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Comment>, i64)>;

    /// Get replies to a comment
    async fn list_replies(&self, parent_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Comment>, i64)>;

    /// Update vote counts
    async fn update_votes(&self, id: Uuid, upvotes: i32, downvotes: i32) -> ApiResult<()>;
}

// ==================== Vote Repository ====================

#[async_trait]
pub trait VoteRepository: Send + Sync {
    /// Create or update vote
    async fn upsert(&self, identity_id: Uuid, target_type: VoteTargetType, target_id: Uuid, value: i16) -> ApiResult<Vote>;

    /// Delete vote
    async fn delete(&self, identity_id: Uuid, target_type: VoteTargetType, target_id: Uuid) -> ApiResult<()>;

    /// Get vote
    async fn find(&self, identity_id: Uuid, target_type: VoteTargetType, target_id: Uuid) -> ApiResult<Option<Vote>>;

    /// Get votes for multiple targets (for batch loading)
    async fn find_many(&self, identity_id: Uuid, target_type: VoteTargetType, target_ids: &[Uuid]) -> ApiResult<Vec<Vote>>;
}

// ==================== Message Repository ====================

#[async_trait]
pub trait MessageRepository: Send + Sync {
    /// Create conversation
    async fn create_conversation(&self, participant_ids: &[Uuid]) -> ApiResult<Conversation>;

    /// Add participant to conversation
    async fn add_participant(
        &self,
        conversation_id: Uuid,
        identity_id: Uuid,
        encrypted_key: &[u8],
    ) -> ApiResult<ConversationParticipant>;

    /// Find conversation by ID
    async fn find_conversation(&self, id: Uuid) -> ApiResult<Option<Conversation>>;

    /// List conversations for identity
    async fn list_conversations(&self, identity_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Conversation>, i64)>;

    /// Get conversation participants
    async fn get_participants(&self, conversation_id: Uuid) -> ApiResult<Vec<ConversationParticipant>>;

    /// Create message
    async fn create_message(
        &self,
        conversation_id: Uuid,
        sender_id: Uuid,
        encrypted_content: &[u8],
        nonce: &[u8],
    ) -> ApiResult<Message>;

    /// List messages in conversation
    async fn list_messages(&self, conversation_id: Uuid, pagination: &PaginationParams) -> ApiResult<(Vec<Message>, i64)>;

    /// Mark messages as read
    async fn mark_read(&self, conversation_id: Uuid, identity_id: Uuid) -> ApiResult<()>;

    /// Check if identity is participant
    async fn is_participant(&self, conversation_id: Uuid, identity_id: Uuid) -> ApiResult<bool>;
}

// ==================== Media Repository ====================

#[async_trait]
pub trait MediaRepository: Send + Sync {
    /// Create media record
    async fn create(
        &self,
        uploader_id: Uuid,
        file_hash: &str,
        mime_type: &str,
        file_size: i32,
        storage_path: &str,
        thumbnail_path: Option<&str>,
    ) -> ApiResult<Media>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Media>>;

    /// Find by hash (for deduplication)
    async fn find_by_hash(&self, hash: &str) -> ApiResult<Option<Media>>;

    /// Mark as processed
    async fn mark_processed(&self, id: Uuid) -> ApiResult<()>;

    /// Delete media record
    async fn delete(&self, id: Uuid) -> ApiResult<()>;
}

// ==================== Report Repository ====================

#[async_trait]
pub trait ReportRepository: Send + Sync {
    /// Create report
    async fn create(&self, reporter_id: Uuid, request: &CreateReportRequest) -> ApiResult<Report>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Report>>;

    /// List reports (for moderators)
    async fn list(
        &self,
        status: Option<ReportStatus>,
        target_type: Option<ReportTargetType>,
        pagination: &PaginationParams,
    ) -> ApiResult<(Vec<Report>, i64)>;

    /// Update report status
    async fn update_status(
        &self,
        id: Uuid,
        status: ReportStatus,
        reviewed_by: Uuid,
        notes: Option<&str>,
    ) -> ApiResult<Report>;
}

// ==================== Notification Repository ====================

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    /// Create notification
    async fn create(
        &self,
        recipient_id: Uuid,
        notification_type: NotificationType,
        payload: serde_json::Value,
    ) -> ApiResult<Notification>;

    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> ApiResult<Option<Notification>>;

    /// List notifications for recipient
    async fn list_by_recipient(
        &self,
        recipient_id: Uuid,
        unread_only: bool,
        pagination: &PaginationParams,
    ) -> ApiResult<(Vec<Notification>, i64)>;

    /// Mark as read
    async fn mark_read(&self, id: Uuid) -> ApiResult<()>;

    /// Mark all as read for recipient
    async fn mark_all_read(&self, recipient_id: Uuid) -> ApiResult<()>;

    /// Count unread
    async fn count_unread(&self, recipient_id: Uuid) -> ApiResult<i64>;

    /// Delete old notifications
    async fn delete_old(&self, days: i32) -> ApiResult<u64>;
}
