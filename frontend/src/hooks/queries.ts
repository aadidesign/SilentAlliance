import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import type { PostSort, TimeRange } from '@/types';

// ==================== Feed ====================

export function useFeed(sort: PostSort = 'hot', limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['feed', sort, limit, offset],
    queryFn: () => api.getFeed(sort, limit, offset),
  });
}

export function useAllFeed(sort: PostSort = 'hot', limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['feed', 'all', sort, limit, offset],
    queryFn: () => api.getAllFeed(sort, limit, offset),
  });
}

export function usePopularFeed(limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['feed', 'popular', limit, offset],
    queryFn: () => api.getPopularFeed(limit, offset),
  });
}

// ==================== Posts ====================

export function usePost(id: string) {
  return useQuery({
    queryKey: ['post', id],
    queryFn: () => api.getPost(id),
    enabled: !!id,
  });
}

export function usePostComments(postId: string, limit = 50, offset = 0) {
  return useQuery({
    queryKey: ['post', postId, 'comments', limit, offset],
    queryFn: () => api.getPostComments(postId, limit, offset),
    enabled: !!postId,
  });
}

// ==================== Spaces ====================

export function useSpaces(limit = 100, offset = 0) {
  return useQuery({
    queryKey: ['spaces', limit, offset],
    queryFn: () => api.listSpaces(limit, offset),
  });
}

export function useSpace(slug: string) {
  return useQuery({
    queryKey: ['space', slug],
    queryFn: () => api.getSpace(slug),
    enabled: !!slug,
  });
}

export function useSpacePosts(
  slug: string,
  sort: PostSort = 'hot',
  limit = 25,
  offset = 0,
  timeRange: TimeRange = 'all'
) {
  return useQuery({
    queryKey: ['space', slug, 'posts', sort, limit, offset, timeRange],
    queryFn: () => api.getSpacePosts(slug, sort, limit, offset, timeRange),
    enabled: !!slug,
  });
}

export function useSpaceMembers(slug: string, limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['space', slug, 'members', limit, offset],
    queryFn: () => api.getSpaceMembers(slug, limit, offset),
    enabled: !!slug,
  });
}

// ==================== Identity ====================

export function useIdentity(id: string) {
  return useQuery({
    queryKey: ['identity', id],
    queryFn: () => api.getIdentity(id),
    enabled: !!id,
  });
}

export function useIdentityPosts(id: string, limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['identity', id, 'posts', limit, offset],
    queryFn: () => api.getIdentityPosts(id, limit, offset),
    enabled: !!id,
  });
}

export function useIdentityComments(id: string, limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['identity', id, 'comments', limit, offset],
    queryFn: () => api.getIdentityComments(id, limit, offset),
    enabled: !!id,
  });
}

// ==================== Notifications ====================

export function useNotifications(limit = 25, offset = 0) {
  return useQuery({
    queryKey: ['notifications', limit, offset],
    queryFn: () => api.listNotifications(limit, offset),
  });
}

export function useUnreadCount() {
  return useQuery({
    queryKey: ['notifications', 'unread-count'],
    queryFn: () => api.getUnreadCount(),
    refetchInterval: 30000, // poll every 30s
  });
}

// ==================== Messages ====================

export function useConversations() {
  return useQuery({
    queryKey: ['conversations'],
    queryFn: () => api.listConversations(),
  });
}

export function useConversation(id: string) {
  return useQuery({
    queryKey: ['conversation', id],
    queryFn: () => api.getConversation(id),
    enabled: !!id,
  });
}

export function useMessages(conversationId: string, limit = 50, offset = 0) {
  return useQuery({
    queryKey: ['conversation', conversationId, 'messages', limit, offset],
    queryFn: () => api.getMessages(conversationId, limit, offset),
    enabled: !!conversationId,
  });
}

// ==================== Me ====================

export function useMe() {
  return useQuery({
    queryKey: ['me'],
    queryFn: () => api.getMe(),
  });
}
