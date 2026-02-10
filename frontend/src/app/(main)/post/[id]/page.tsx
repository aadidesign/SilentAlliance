'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { motion } from 'framer-motion';
import {
  ArrowLeft,
  MessageSquare,
  Share2,
  Bookmark,
  Pin,
  Lock,
  ExternalLink,
  Flag,
  Clock,
  Award,
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { VoteButton } from '@/components/ui/VoteButton';
import { Textarea } from '@/components/ui/Input';
import { Tabs } from '@/components/ui/Tabs';
import { Skeleton } from '@/components/ui/Skeleton';
import { CommentThread } from '@/components/comment/CommentThread';
import { formatTimeAgo, formatNumber } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { useAuthStore } from '@/lib/store';
import { usePost, usePostComments } from '@/hooks/queries';
import { useVotePost, useVoteComment, useCreateComment } from '@/hooks/mutations';

const commentSortTabs = [
  { id: 'best', label: 'Best', icon: <Award size={14} /> },
  { id: 'new', label: 'New', icon: <Clock size={14} /> },
];

export default function PostDetailPage() {
  const params = useParams();
  const router = useRouter();
  const postId = params.id as string;
  const { isAuthenticated } = useAuthStore();
  const [commentSort, setCommentSort] = useState('best');
  const [newComment, setNewComment] = useState('');

  const { data: post, isLoading: postLoading } = usePost(postId);
  const { data: commentsData, isLoading: commentsLoading } = usePostComments(postId);
  const voteMutation = useVotePost();
  const voteCommentMutation = useVoteComment();
  const createCommentMutation = useCreateComment(postId);

  const comments = commentsData?.data ?? [];

  if (postLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="w-20 h-4" />
        <Card padding="lg" className="space-y-4">
          <Skeleton className="w-48 h-4" />
          <Skeleton className="w-full h-8" />
          <Skeleton className="w-full h-32" />
          <Skeleton className="w-48 h-8" />
        </Card>
      </div>
    );
  }

  if (!post) {
    return (
      <div className="text-center py-20">
        <MessageSquare size={32} className="mx-auto text-text-tertiary mb-3" />
        <h2 className="text-lg font-semibold text-text-primary mb-1">Post not found</h2>
        <p className="text-sm text-text-tertiary mb-4">This post may have been deleted or doesn&apos;t exist.</p>
        <Button variant="secondary" onClick={() => router.push('/feed')}>
          Back to Feed
        </Button>
      </div>
    );
  }

  const handleSubmitComment = () => {
    if (!newComment.trim()) return;
    createCommentMutation.mutate(
      { content: newComment },
      { onSuccess: () => setNewComment('') }
    );
  };

  return (
    <div className="space-y-4">
      {/* Back button */}
      <button
        onClick={() => router.back()}
        className="flex items-center gap-1.5 text-sm text-text-tertiary hover:text-text-secondary transition-colors -mb-1"
      >
        <ArrowLeft size={16} />
        Back
      </button>

      {/* Post */}
      <motion.div
        initial={pageEntrance.initial}
        animate={pageEntrance.animate}
        transition={pageEntrance.transition}
      >
        <Card padding="lg">
          {/* Meta */}
          <div className="flex items-center gap-2 text-xs text-text-tertiary mb-4">
            <Link
              href={`/s/${post.space?.slug}`}
              className="flex items-center gap-1.5 font-medium text-text-secondary hover:text-accent transition-colors"
            >
              <div className="w-6 h-6 rounded-md bg-surface-hover flex items-center justify-center">
                <span className="text-2xs font-bold">
                  {post.space?.name.charAt(0).toUpperCase()}
                </span>
              </div>
              s/{post.space?.name}
            </Link>
            <span>&bull;</span>
            <div className="flex items-center gap-1.5">
              <Avatar id={post.author_id || ''} name={post.author?.display_name} size="xs" />
              <Link
                href={`/u/${post.author_id}`}
                className="hover:text-text-secondary transition-colors"
              >
                {post.author?.display_name || 'Anonymous'}
              </Link>
            </div>
            <span>&bull;</span>
            <time>{formatTimeAgo(post.created_at)}</time>
            {post.is_pinned && (
              <Badge variant="accent">
                <Pin size={10} className="mr-1" /> Pinned
              </Badge>
            )}
            {post.is_locked && (
              <Badge variant="warning">
                <Lock size={10} className="mr-1" /> Locked
              </Badge>
            )}
          </div>

          {/* Title */}
          <h1 className="text-xl sm:text-2xl font-bold text-text-primary mb-4 leading-tight">
            {post.title}
          </h1>

          {/* Content */}
          {post.content && (
            <div className="prose-dark mb-6 whitespace-pre-wrap">
              {post.content}
            </div>
          )}

          {/* Link */}
          {post.content_type === 'link' && post.url && (
            <a
              href={post.url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 p-3 bg-bg-tertiary rounded-xl border border-surface-border text-sm text-accent hover:text-accent-hover transition-colors mb-6"
            >
              <ExternalLink size={16} />
              {post.url}
            </a>
          )}

          {/* Actions */}
          <div className="flex items-center gap-2 pt-2 border-t border-surface-border">
            <VoteButton
              score={post.score}
              userVote={post.user_vote}
              onVote={(value) => voteMutation.mutate({ id: post.id, value })}
            />
            <div className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-text-tertiary">
              <MessageSquare size={14} />
              {formatNumber(post.comment_count)} comments
            </div>
            <button className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
              <Share2 size={14} />
              Share
            </button>
            <button className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
              <Bookmark size={14} />
              Save
            </button>
            <button className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
              <Flag size={14} />
              Report
            </button>
          </div>
        </Card>
      </motion.div>

      {/* Comment form */}
      {isAuthenticated && !post.is_locked && (
        <Card>
          <Textarea
            placeholder="What are your thoughts?"
            value={newComment}
            onChange={(e) => setNewComment(e.target.value)}
            className="min-h-[100px]"
          />
          <div className="flex justify-end mt-3">
            <Button
              size="sm"
              disabled={!newComment.trim() || createCommentMutation.isPending}
              isLoading={createCommentMutation.isPending}
              onClick={handleSubmitComment}
            >
              Comment
            </Button>
          </div>
        </Card>
      )}

      {post.is_locked && (
        <Card className="flex items-center gap-3 text-sm text-text-tertiary">
          <Lock size={16} />
          This post has been locked. New comments are disabled.
        </Card>
      )}

      {/* Comments */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <Tabs
            tabs={commentSortTabs}
            activeTab={commentSort}
            onChange={setCommentSort}
          />
        </div>

        <Card padding="sm">
          {commentsLoading ? (
            <div className="space-y-4 p-4">
              {Array.from({ length: 3 }).map((_, i) => (
                <div key={i} className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Skeleton className="w-6 h-6 rounded-full" />
                    <Skeleton className="w-24 h-3" />
                  </div>
                  <Skeleton className="w-full h-12" />
                </div>
              ))}
            </div>
          ) : comments.length > 0 ? (
            <CommentThread
              comments={comments}
              onVote={(id, value) => voteCommentMutation.mutate({ id, value })}
              onReply={(parentId, content) =>
                createCommentMutation.mutate({ content, parent_id: parentId })
              }
            />
          ) : (
            <div className="py-12 text-center">
              <MessageSquare size={24} className="mx-auto text-text-tertiary mb-2" />
              <p className="text-sm text-text-tertiary">No comments yet. Start the conversation!</p>
            </div>
          )}
        </Card>
      </div>
    </div>
  );
}
