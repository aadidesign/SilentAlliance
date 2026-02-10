import { useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/lib/api';
import toast from 'react-hot-toast';

// ==================== Voting ====================

export function useVotePost() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, value }: { id: string; value: number }) =>
      api.votePost(id, value),
    onSuccess: (_data, variables) => {
      qc.invalidateQueries({ queryKey: ['post', variables.id] });
      qc.invalidateQueries({ queryKey: ['feed'] });
      qc.invalidateQueries({ queryKey: ['space'] });
    },
    onError: () => toast.error('Failed to vote'),
  });
}

export function useUnvotePost() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.unvotePost(id),
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: ['post', id] });
      qc.invalidateQueries({ queryKey: ['feed'] });
    },
    onError: () => toast.error('Failed to remove vote'),
  });
}

export function useVoteComment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, value }: { id: string; value: number }) =>
      api.voteComment(id, value),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['post'] });
    },
    onError: () => toast.error('Failed to vote'),
  });
}

// ==================== Comments ====================

export function useCreateComment(postId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { content: string; parent_id?: string }) =>
      api.createComment(postId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['post', postId, 'comments'] });
      qc.invalidateQueries({ queryKey: ['post', postId] });
      toast.success('Comment posted');
    },
    onError: () => toast.error('Failed to post comment'),
  });
}

// ==================== Posts ====================

export function useCreatePost(slug: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { title: string; content?: string; content_type?: string; url?: string }) =>
      api.createPost(slug, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['space', slug, 'posts'] });
      qc.invalidateQueries({ queryKey: ['feed'] });
      toast.success('Post created');
    },
    onError: () => toast.error('Failed to create post'),
  });
}

export function useDeletePost() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deletePost(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['feed'] });
      qc.invalidateQueries({ queryKey: ['space'] });
      toast.success('Post deleted');
    },
    onError: () => toast.error('Failed to delete post'),
  });
}

// ==================== Spaces ====================

export function useJoinSpace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (slug: string) => api.joinSpace(slug),
    onSuccess: (_data, slug) => {
      qc.invalidateQueries({ queryKey: ['space', slug] });
      qc.invalidateQueries({ queryKey: ['spaces'] });
      toast.success('Joined space');
    },
    onError: () => toast.error('Failed to join space'),
  });
}

export function useLeaveSpace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (slug: string) => api.leaveSpace(slug),
    onSuccess: (_data, slug) => {
      qc.invalidateQueries({ queryKey: ['space', slug] });
      qc.invalidateQueries({ queryKey: ['spaces'] });
      toast.success('Left space');
    },
    onError: () => toast.error('Failed to leave space'),
  });
}

export function useCreateSpace() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { name: string; description?: string; is_private?: boolean }) =>
      api.createSpace(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['spaces'] });
      toast.success('Space created');
    },
    onError: () => toast.error('Failed to create space'),
  });
}

// ==================== Messages ====================

export function useSendMessage(conversationId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { encrypted_content: string; nonce: string }) =>
      api.sendMessage(conversationId, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['conversation', conversationId, 'messages'] });
      qc.invalidateQueries({ queryKey: ['conversations'] });
    },
    onError: () => toast.error('Failed to send message'),
  });
}

export function useCreateConversation() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: {
      participant_ids: string[];
      initial_message?: { encrypted_content: string; nonce: string };
    }) => api.createConversation(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['conversations'] });
    },
    onError: () => toast.error('Failed to create conversation'),
  });
}

// ==================== Notifications ====================

export function useMarkNotificationRead() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.markNotificationRead(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notifications'] });
    },
  });
}

export function useMarkAllNotificationsRead() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => api.markAllNotificationsRead(),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['notifications'] });
      toast.success('All notifications marked as read');
    },
    onError: () => toast.error('Failed to mark notifications as read'),
  });
}

// ==================== Profile ====================

export function useUpdateProfile() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (data: { display_name?: string; bio?: string }) =>
      api.updateMe(data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['me'] });
      toast.success('Profile updated');
    },
    onError: () => toast.error('Failed to update profile'),
  });
}
