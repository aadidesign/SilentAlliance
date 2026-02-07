// ==================== Identity ====================

export interface Identity {
  id: string;
  public_key_fingerprint: string;
  display_name: string | null;
  avatar_hash: string | null;
  bio: string | null;
  karma: number;
  is_verified: boolean;
  is_suspended: boolean;
  suspended_reason: string | null;
  suspended_until: string | null;
  created_at: string;
  updated_at: string;
}

export interface IdentityPublic {
  id: string;
  public_key_fingerprint: string;
  display_name: string | null;
  avatar_hash: string | null;
  bio: string | null;
  karma: number;
  is_verified: boolean;
  created_at: string;
}

// ==================== Space ====================

export interface Space {
  id: string;
  name: string;
  slug: string;
  description: string | null;
  rules: string[];
  icon_url: string | null;
  banner_url: string | null;
  is_private: boolean;
  is_nsfw: boolean;
  creator_id: string | null;
  subscriber_count: number;
  post_count: number;
  created_at: string;
  updated_at: string;
}

export interface SpaceSummary {
  id: string;
  name: string;
  slug: string;
  icon_url: string | null;
  subscriber_count: number;
}

export type MemberRole = 'member' | 'moderator' | 'admin';

export interface SpaceMember {
  id: string;
  space_id: string;
  identity_id: string;
  role: MemberRole;
  joined_at: string;
}

// ==================== Post ====================

export type ContentType = 'text' | 'link' | 'media' | 'poll';

export interface Post {
  id: string;
  space_id: string;
  author_id: string | null;
  title: string;
  content: string | null;
  content_type: ContentType;
  url: string | null;
  media_ids: string[];
  upvotes: number;
  downvotes: number;
  score: number;
  comment_count: number;
  is_pinned: boolean;
  is_locked: boolean;
  is_removed: boolean;
  removed_reason: string | null;
  created_at: string;
  updated_at: string;
}

export interface PostWithContext extends Post {
  author: IdentityPublic | null;
  space: SpaceSummary | null;
  user_vote: number | null;
}

// ==================== Comment ====================

export interface Comment {
  id: string;
  post_id: string;
  parent_id: string | null;
  author_id: string | null;
  content: string;
  depth: number;
  path: string;
  upvotes: number;
  downvotes: number;
  score: number;
  is_removed: boolean;
  removed_reason: string | null;
  created_at: string;
  updated_at: string;
}

export interface CommentWithContext extends Comment {
  author: IdentityPublic | null;
  user_vote: number | null;
  replies: CommentWithContext[];
}

// ==================== Vote ====================

export type VoteTargetType = 'post' | 'comment';

// ==================== Messages ====================

export interface Conversation {
  id: string;
  participants: ConversationParticipant[];
  last_message?: MessageResponse;
  created_at: string;
  updated_at: string;
}

export interface ConversationParticipant {
  id: string;
  conversation_id: string;
  identity_id: string;
  identity?: IdentityPublic;
  last_read_at: string | null;
  created_at: string;
}

export interface MessageResponse {
  id: string;
  conversation_id: string;
  sender_id: string | null;
  sender: IdentityPublic | null;
  encrypted_content: string;
  nonce: string;
  created_at: string;
}

// ==================== Notifications ====================

export type NotificationType =
  | 'post_reply'
  | 'comment_reply'
  | 'mention'
  | 'new_message'
  | 'space_invite'
  | 'moderator_action'
  | 'system_alert';

export interface Notification {
  id: string;
  recipient_id: string;
  notification_type: NotificationType;
  payload: Record<string, unknown>;
  is_read: boolean;
  created_at: string;
}

// ==================== Pagination ====================

export interface PaginationInfo {
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

export interface PaginatedResponse<T> {
  data: T[];
  pagination: PaginationInfo;
}

// ==================== Feed ====================

export type PostSort = 'hot' | 'new' | 'top' | 'rising' | 'controversial';
export type TimeRange = 'hour' | 'day' | 'week' | 'month' | 'year' | 'all';

// ==================== Auth ====================

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface ChallengeResponse {
  challenge: string;
  expires_at: string;
}

// ==================== Reports ====================

export type ReportTargetType = 'post' | 'comment' | 'message' | 'identity' | 'space';
export type ReportReason =
  | 'spam'
  | 'harassment'
  | 'hate_speech'
  | 'violence'
  | 'misinformation'
  | 'illegal_content'
  | 'privacy_violation'
  | 'impersonation'
  | 'other';
export type ReportStatus = 'pending' | 'reviewed' | 'actioned' | 'dismissed';
