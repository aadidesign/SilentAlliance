//! E2E Encrypted Messages handlers

use axum::{extract::{Path, State}, http::StatusCode, Json};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::extractors::Pagination;
use crate::domain::entities::*;
use crate::errors::{ApiError, ApiResult};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

/// List conversations for the current user
pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<ConversationSummary>>> {
    let conversations = sqlx::query_as!(
        ConversationSummary,
        r#"
        SELECT c.id, c.created_at, c.updated_at,
               (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id) as "message_count!",
               (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id AND m.created_at > COALESCE(cp.last_read_at, '1970-01-01')) as "unread_count!"
        FROM conversations c
        JOIN conversation_participants cp ON cp.conversation_id = c.id
        WHERE cp.identity_id = $1
        ORDER BY c.updated_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user.identity_id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM conversation_participants WHERE identity_id = $1",
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    Ok(Json(PaginatedResponse {
        data: conversations,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Create a new conversation
pub async fn create_conversation(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(request): Json<CreateConversationRequest>,
) -> ApiResult<(StatusCode, Json<Conversation>)> {
    if request.participant_ids.is_empty() {
        return Err(ApiError::InvalidInput("At least one participant required".to_string()));
    }

    // Include the creator as a participant
    let mut all_participants = request.participant_ids.clone();
    if !all_participants.contains(&user.identity_id) {
        all_participants.push(user.identity_id);
    }

    // Verify all participants exist
    let existing_count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM identities WHERE id = ANY($1)",
        &all_participants
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(0);

    if existing_count != all_participants.len() as i64 {
        return Err(ApiError::NotFound("One or more participants not found".to_string()));
    }

    let conv_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Create conversation
    let conversation = sqlx::query_as!(
        Conversation,
        "INSERT INTO conversations (id, created_at, updated_at) VALUES ($1, $2, $2) RETURNING id, created_at, updated_at",
        conv_id,
        now
    )
    .fetch_one(state.db.pool())
    .await?;

    // Add participants (with placeholder encrypted keys - client generates real keys)
    for participant_id in &all_participants {
        sqlx::query!(
            "INSERT INTO conversation_participants (id, conversation_id, identity_id, encrypted_key, created_at) VALUES ($1, $2, $3, $4, $5)",
            Uuid::new_v4(),
            conv_id,
            participant_id,
            vec![0u8; 32], // Placeholder - client provides real encrypted key
            now
        )
        .execute(state.db.pool())
        .await?;
    }

    // Send initial message if provided
    if let Some(msg) = request.initial_message {
        let encrypted_content = BASE64.decode(&msg.encrypted_content)
            .map_err(|_| ApiError::InvalidInput("Invalid base64 content".to_string()))?;
        let nonce = BASE64.decode(&msg.nonce)
            .map_err(|_| ApiError::InvalidInput("Invalid base64 nonce".to_string()))?;

        sqlx::query!(
            "INSERT INTO messages (id, conversation_id, sender_id, encrypted_content, nonce, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
            Uuid::new_v4(),
            conv_id,
            user.identity_id,
            encrypted_content,
            nonce,
            now
        )
        .execute(state.db.pool())
        .await?;
    }

    Ok((StatusCode::CREATED, Json(conversation)))
}

/// Get conversation details
pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<Json<ConversationDetail>> {
    // Verify user is participant
    let is_participant = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = $1 AND identity_id = $2)",
        id,
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(false);

    if !is_participant {
        return Err(ApiError::Forbidden);
    }

    let conversation = sqlx::query_as!(
        Conversation,
        "SELECT id, created_at, updated_at FROM conversations WHERE id = $1",
        id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Conversation not found".to_string()))?;

    let participants = sqlx::query!(
        r#"
        SELECT cp.identity_id, cp.encrypted_key, i.display_name, i.public_key_fingerprint
        FROM conversation_participants cp
        JOIN identities i ON i.id = cp.identity_id
        WHERE cp.conversation_id = $1
        "#,
        id
    )
    .fetch_all(state.db.pool())
    .await?;

    Ok(Json(ConversationDetail {
        id: conversation.id,
        created_at: conversation.created_at,
        participants: participants
            .into_iter()
            .map(|p| ParticipantInfo {
                identity_id: p.identity_id,
                display_name: p.display_name,
                fingerprint: p.public_key_fingerprint,
                encrypted_key: BASE64.encode(&p.encrypted_key),
            })
            .collect(),
    }))
}

/// List messages in a conversation
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    Pagination(pagination): Pagination,
) -> ApiResult<Json<PaginatedResponse<MessageResponse>>> {
    // Verify user is participant
    let is_participant = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = $1 AND identity_id = $2)",
        id,
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(false);

    if !is_participant {
        return Err(ApiError::Forbidden);
    }

    let messages = sqlx::query!(
        r#"
        SELECT m.id, m.conversation_id, m.sender_id, m.encrypted_content, m.nonce, m.created_at
        FROM messages m
        WHERE m.conversation_id = $1
        ORDER BY m.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        id,
        pagination.limit,
        pagination.offset
    )
    .fetch_all(state.db.pool())
    .await?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM messages WHERE conversation_id = $1", id)
        .fetch_one(state.db.pool())
        .await?
        .unwrap_or(0);

    let message_responses: Vec<MessageResponse> = messages
        .into_iter()
        .map(|m| MessageResponse {
            id: m.id,
            conversation_id: m.conversation_id,
            sender_id: m.sender_id,
            sender: None,
            encrypted_content: BASE64.encode(&m.encrypted_content),
            nonce: BASE64.encode(&m.nonce),
            created_at: m.created_at,
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: message_responses,
        pagination: PaginationInfo::new(total, pagination.limit, pagination.offset),
    }))
}

/// Send a message
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
    Json(request): Json<EncryptedMessageRequest>,
) -> ApiResult<(StatusCode, Json<MessageResponse>)> {
    // Verify user is participant
    let is_participant = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM conversation_participants WHERE conversation_id = $1 AND identity_id = $2)",
        id,
        user.identity_id
    )
    .fetch_one(state.db.pool())
    .await?
    .unwrap_or(false);

    if !is_participant {
        return Err(ApiError::Forbidden);
    }

    let encrypted_content = BASE64.decode(&request.encrypted_content)
        .map_err(|_| ApiError::InvalidInput("Invalid base64 content".to_string()))?;
    let nonce = BASE64.decode(&request.nonce)
        .map_err(|_| ApiError::InvalidInput("Invalid base64 nonce".to_string()))?;

    let msg_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query!(
        "INSERT INTO messages (id, conversation_id, sender_id, encrypted_content, nonce, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
        msg_id,
        id,
        user.identity_id,
        encrypted_content,
        nonce,
        now
    )
    .execute(state.db.pool())
    .await?;

    // Update conversation timestamp
    sqlx::query!("UPDATE conversations SET updated_at = $1 WHERE id = $2", now, id)
        .execute(state.db.pool())
        .await?;

    Ok((StatusCode::CREATED, Json(MessageResponse {
        id: msg_id,
        conversation_id: id,
        sender_id: Some(user.identity_id),
        sender: None,
        encrypted_content: request.encrypted_content,
        nonce: request.nonce,
        created_at: now,
    })))
}

/// Mark messages as read
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    user: AuthenticatedUser,
) -> ApiResult<StatusCode> {
    sqlx::query!(
        "UPDATE conversation_participants SET last_read_at = NOW() WHERE conversation_id = $1 AND identity_id = $2",
        id,
        user.identity_id
    )
    .execute(state.db.pool())
    .await?;

    Ok(StatusCode::OK)
}

/// Get public key for encryption
pub async fn get_public_key(
    State(state): State<Arc<AppState>>,
    Path(identity_id): Path<Uuid>,
    _user: AuthenticatedUser,
) -> ApiResult<Json<PublicKeyResponse>> {
    let identity = sqlx::query!(
        "SELECT public_key, public_key_fingerprint FROM identities WHERE id = $1",
        identity_id
    )
    .fetch_optional(state.db.pool())
    .await?
    .ok_or_else(|| ApiError::NotFound("Identity not found".to_string()))?;

    Ok(Json(PublicKeyResponse {
        identity_id,
        public_key: BASE64.encode(&identity.public_key),
        fingerprint: identity.public_key_fingerprint,
    }))
}

// Response types

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ConversationSummary {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub message_count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationDetail {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub participants: Vec<ParticipantInfo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticipantInfo {
    pub identity_id: Uuid,
    pub display_name: Option<String>,
    pub fingerprint: String,
    pub encrypted_key: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyResponse {
    pub identity_id: Uuid,
    pub public_key: String,
    pub fingerprint: String,
}
