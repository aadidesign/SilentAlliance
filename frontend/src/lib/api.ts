import { useAuthStore } from './store';
import type {
  AuthTokens,
  ChallengeResponse,
  Conversation,
  CommentWithContext,
  Identity,
  MessageResponse,
  Notification,
  PaginatedResponse,
  Post,
  PostSort,
  PostWithContext,
  RegisterResponse,
  Space,
  SpaceMember,
  TimeRange,
} from '@/types';

const API_BASE = process.env.NEXT_PUBLIC_API_URL || '';
const API_V1 = `${API_BASE}/api/v1`;

// ==================== HTTP Client ====================

class ApiClient {
  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const { tokens, logout } = useAuthStore.getState();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers as Record<string, string>),
    };

    if (tokens?.access_token) {
      headers['Authorization'] = `Bearer ${tokens.access_token}`;
    }

    const response = await fetch(`${API_V1}${path}`, {
      ...options,
      headers,
    });

    if (response.status === 401 && tokens?.refresh_token) {
      // Try refresh
      const refreshed = await this.refreshToken(tokens.refresh_token);
      if (refreshed) {
        headers['Authorization'] = `Bearer ${refreshed.access_token}`;
        const retryResponse = await fetch(`${API_V1}${path}`, {
          ...options,
          headers,
        });
        if (!retryResponse.ok) {
          throw new ApiError(retryResponse.status, await retryResponse.text());
        }
        return retryResponse.json();
      } else {
        logout();
        throw new ApiError(401, 'Session expired');
      }
    }

    if (!response.ok) {
      throw new ApiError(response.status, await response.text());
    }

    if (response.status === 204) return {} as T;
    return response.json();
  }

  private async refreshToken(refreshToken: string): Promise<AuthTokens | null> {
    try {
      const response = await fetch(`${API_V1}/auth/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refresh_token: refreshToken }),
      });
      if (!response.ok) return null;
      const tokens = await response.json();
      useAuthStore.getState().setTokens(tokens);
      return tokens;
    } catch {
      return null;
    }
  }

  // ==================== Auth ====================

  async register(publicKey: string, displayName?: string): Promise<RegisterResponse> {
    return this.request<RegisterResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ public_key: publicKey, display_name: displayName }),
    });
  }

  async getChallenge(fingerprint: string): Promise<ChallengeResponse> {
    return this.request<ChallengeResponse>('/auth/challenge', {
      method: 'POST',
      body: JSON.stringify({ fingerprint }),
    });
  }

  async login(fingerprint: string, challenge: string, signature: string): Promise<AuthTokens> {
    return this.request<AuthTokens>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ fingerprint, challenge, signature }),
    });
  }

  async logout(): Promise<void> {
    return this.request('/auth/logout', { method: 'POST' });
  }

  // ==================== Identity ====================

  async getMe(): Promise<Identity> {
    return this.request<Identity>('/identity/me');
  }

  async updateMe(data: { display_name?: string; bio?: string }): Promise<Identity> {
    return this.request<Identity>('/identity/me', {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async getIdentity(id: string): Promise<Identity> {
    return this.request<Identity>(`/identity/${id}`);
  }

  async getIdentityPosts(id: string, limit = 25, offset = 0): Promise<PaginatedResponse<Post>> {
    return this.request(`/identity/${id}/posts?limit=${limit}&offset=${offset}`);
  }

  async getIdentityComments(id: string, limit = 25, offset = 0): Promise<PaginatedResponse<CommentWithContext>> {
    return this.request(`/identity/${id}/comments?limit=${limit}&offset=${offset}`);
  }

  // ==================== Spaces ====================

  async listSpaces(limit = 25, offset = 0): Promise<PaginatedResponse<Space>> {
    return this.request(`/spaces?limit=${limit}&offset=${offset}`);
  }

  async getSpace(slug: string): Promise<Space> {
    return this.request<Space>(`/spaces/${slug}`);
  }

  async createSpace(data: { name: string; description?: string; is_private?: boolean }): Promise<Space> {
    return this.request<Space>('/spaces', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async joinSpace(slug: string): Promise<void> {
    return this.request(`/spaces/${slug}/join`, { method: 'POST' });
  }

  async leaveSpace(slug: string): Promise<void> {
    return this.request(`/spaces/${slug}/leave`, { method: 'POST' });
  }

  async getSpaceMembers(slug: string, limit = 25, offset = 0): Promise<PaginatedResponse<SpaceMember>> {
    return this.request(`/spaces/${slug}/members?limit=${limit}&offset=${offset}`);
  }

  // ==================== Posts ====================

  async getSpacePosts(
    slug: string,
    sort: PostSort = 'hot',
    limit = 25,
    offset = 0,
    timeRange: TimeRange = 'all'
  ): Promise<PaginatedResponse<PostWithContext>> {
    return this.request(
      `/spaces/${slug}/posts?sort=${sort}&limit=${limit}&offset=${offset}&time_range=${timeRange}`
    );
  }

  async getPost(id: string): Promise<PostWithContext> {
    return this.request<PostWithContext>(`/posts/${id}`);
  }

  async createPost(slug: string, data: {
    title: string;
    content?: string;
    content_type?: string;
    url?: string;
  }): Promise<Post> {
    return this.request<Post>(`/spaces/${slug}/posts`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updatePost(id: string, data: { content?: string }): Promise<Post> {
    return this.request<Post>(`/posts/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async deletePost(id: string): Promise<void> {
    return this.request(`/posts/${id}`, { method: 'DELETE' });
  }

  async votePost(id: string, value: number): Promise<void> {
    return this.request(`/posts/${id}/vote`, {
      method: 'POST',
      body: JSON.stringify({ value }),
    });
  }

  async unvotePost(id: string): Promise<void> {
    return this.request(`/posts/${id}/vote`, { method: 'DELETE' });
  }

  // ==================== Comments ====================

  async getPostComments(postId: string, limit = 50, offset = 0): Promise<PaginatedResponse<CommentWithContext>> {
    return this.request(`/posts/${postId}/comments?limit=${limit}&offset=${offset}`);
  }

  async createComment(postId: string, data: { content: string; parent_id?: string }): Promise<CommentWithContext> {
    return this.request(`/posts/${postId}/comments`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async voteComment(id: string, value: number): Promise<void> {
    return this.request(`/comments/${id}/vote`, {
      method: 'POST',
      body: JSON.stringify({ value }),
    });
  }

  // ==================== Feed ====================

  async getFeed(sort: PostSort = 'hot', limit = 25, offset = 0): Promise<PaginatedResponse<PostWithContext>> {
    return this.request(`/feed?sort=${sort}&limit=${limit}&offset=${offset}`);
  }

  async getAllFeed(sort: PostSort = 'hot', limit = 25, offset = 0): Promise<PaginatedResponse<PostWithContext>> {
    return this.request(`/feed/all?sort=${sort}&limit=${limit}&offset=${offset}`);
  }

  async getPopularFeed(limit = 25, offset = 0): Promise<PaginatedResponse<PostWithContext>> {
    return this.request(`/feed/popular?limit=${limit}&offset=${offset}`);
  }

  // ==================== Messages ====================

  async listConversations(): Promise<PaginatedResponse<Conversation>> {
    return this.request('/messages/conversations');
  }

  async createConversation(data: {
    participant_ids: string[];
    initial_message?: { encrypted_content: string; nonce: string };
  }): Promise<Conversation> {
    return this.request('/messages/conversations', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async getConversation(id: string): Promise<Conversation> {
    return this.request(`/messages/conversations/${id}`);
  }

  async getMessages(conversationId: string, limit = 50, offset = 0): Promise<PaginatedResponse<MessageResponse>> {
    return this.request(`/messages/conversations/${conversationId}/messages?limit=${limit}&offset=${offset}`);
  }

  async sendMessage(conversationId: string, data: { encrypted_content: string; nonce: string }): Promise<MessageResponse> {
    return this.request(`/messages/conversations/${conversationId}/messages`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  // ==================== Notifications ====================

  async listNotifications(limit = 25, offset = 0): Promise<PaginatedResponse<Notification>> {
    return this.request(`/notifications?limit=${limit}&offset=${offset}`);
  }

  async getUnreadCount(): Promise<{ count: number }> {
    return this.request('/notifications/unread-count');
  }

  async markNotificationRead(id: string): Promise<void> {
    return this.request(`/notifications/${id}/read`, { method: 'POST' });
  }

  async markAllNotificationsRead(): Promise<void> {
    return this.request('/notifications/read-all', { method: 'POST' });
  }
}

// ==================== Error Class ====================

export class ApiError extends Error {
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = 'ApiError';
  }
}

// ==================== Singleton ====================

export const api = new ApiClient();
