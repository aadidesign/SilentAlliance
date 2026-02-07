'use client';

import Link from 'next/link';
import { MessageSquare, Share2, Bookmark, MoreHorizontal, Pin, Lock, ExternalLink } from 'lucide-react';
import { cn, formatTimeAgo, formatNumber, truncate } from '@/lib/utils';
import { Avatar } from '@/components/ui/Avatar';
import { Badge } from '@/components/ui/Badge';
import { VoteButton } from '@/components/ui/VoteButton';
import type { PostWithContext } from '@/types';

interface PostCardProps {
  post: PostWithContext;
  onVote?: (value: number) => void;
  compact?: boolean;
}

export function PostCard({ post, onVote, compact = false }: PostCardProps) {
  const authorName = post.author?.display_name || 'Anonymous';
  const spaceName = post.space?.name || 'unknown';
  const spaceSlug = post.space?.slug || '';

  return (
    <article
      className={cn(
        'bg-surface rounded-2xl border border-surface-border',
        'hover:border-surface-hover transition-all duration-200',
        'group'
      )}
    >
      <div className="flex gap-3 p-4">
        {/* Vote column - desktop */}
        {!compact && (
          <div className="hidden sm:flex flex-col items-center pt-1">
            <VoteButton
              score={post.score}
              userVote={post.user_vote}
              onVote={onVote || (() => {})}
              orientation="vertical"
              size="sm"
            />
          </div>
        )}

        {/* Content */}
        <div className="flex-1 min-w-0">
          {/* Meta row */}
          <div className="flex items-center gap-2 text-xs text-text-tertiary mb-2 flex-wrap">
            <Link
              href={`/s/${spaceSlug}`}
              className="flex items-center gap-1.5 font-medium text-text-secondary hover:text-accent transition-colors"
            >
              <div className="w-5 h-5 rounded-md bg-surface-hover flex items-center justify-center">
                <span className="text-2xs font-bold">{spaceName.charAt(0).toUpperCase()}</span>
              </div>
              s/{spaceName}
            </Link>
            <span className="text-text-tertiary/50">&bull;</span>
            <Link
              href={`/u/${post.author_id}`}
              className="hover:text-text-secondary transition-colors"
            >
              {authorName}
            </Link>
            <span className="text-text-tertiary/50">&bull;</span>
            <time>{formatTimeAgo(post.created_at)}</time>
            {post.is_pinned && (
              <Badge variant="accent">
                <Pin size={10} className="mr-1" />
                Pinned
              </Badge>
            )}
            {post.is_locked && (
              <Badge variant="warning">
                <Lock size={10} className="mr-1" />
                Locked
              </Badge>
            )}
          </div>

          {/* Title */}
          <Link href={`/post/${post.id}`}>
            <h2
              className={cn(
                'font-semibold text-text-primary leading-snug mb-1',
                'group-hover:text-accent transition-colors',
                compact ? 'text-sm' : 'text-base'
              )}
            >
              {post.title}
            </h2>
          </Link>

          {/* Content preview */}
          {post.content && !compact && (
            <Link href={`/post/${post.id}`}>
              <p className="text-sm text-text-secondary leading-relaxed mb-3 line-clamp-3">
                {truncate(post.content, 300)}
              </p>
            </Link>
          )}

          {/* Link preview */}
          {post.content_type === 'link' && post.url && (
            <a
              href={post.url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1.5 text-xs text-accent hover:text-accent-hover transition-colors mb-3"
            >
              <ExternalLink size={12} />
              {truncate(post.url, 60)}
            </a>
          )}

          {/* Actions row */}
          <div className="flex items-center gap-1 -ml-1.5 mt-1">
            {/* Mobile vote */}
            <div className="sm:hidden">
              <VoteButton
                score={post.score}
                userVote={post.user_vote}
                onVote={onVote || (() => {})}
                size="sm"
              />
            </div>

            {/* Comments */}
            <Link
              href={`/post/${post.id}`}
              className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors"
            >
              <MessageSquare size={14} />
              <span>{formatNumber(post.comment_count)}</span>
            </Link>

            {/* Share */}
            <button className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
              <Share2 size={14} />
              <span className="hidden sm:inline">Share</span>
            </button>

            {/* Bookmark */}
            <button className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
              <Bookmark size={14} />
              <span className="hidden sm:inline">Save</span>
            </button>

            {/* More */}
            <button className="flex items-center gap-1.5 p-1.5 rounded-lg text-xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors ml-auto">
              <MoreHorizontal size={14} />
            </button>
          </div>
        </div>
      </div>
    </article>
  );
}
