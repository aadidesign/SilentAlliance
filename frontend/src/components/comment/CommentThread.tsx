'use client';

import { useState } from 'react';
import Link from 'next/link';
import { MessageSquare, MoreHorizontal, Flag, ChevronDown, ChevronUp, Reply } from 'lucide-react';
import { cn, formatTimeAgo } from '@/lib/utils';
import { Avatar } from '@/components/ui/Avatar';
import { VoteButton } from '@/components/ui/VoteButton';
import { Button } from '@/components/ui/Button';
import { Textarea } from '@/components/ui/Input';
import type { CommentWithContext } from '@/types';

interface CommentNodeProps {
  comment: CommentWithContext;
  onVote?: (id: string, value: number) => void;
  onReply?: (parentId: string, content: string) => void;
  depth?: number;
}

function CommentNode({ comment, onVote, onReply, depth = 0 }: CommentNodeProps) {
  const [collapsed, setCollapsed] = useState(false);
  const [replying, setReplying] = useState(false);
  const [replyContent, setReplyContent] = useState('');
  const authorName = comment.author?.display_name || 'Anonymous';
  const maxDepth = 6;

  const handleSubmitReply = () => {
    if (replyContent.trim() && onReply) {
      onReply(comment.id, replyContent.trim());
      setReplyContent('');
      setReplying(false);
    }
  };

  return (
    <div
      className={cn(
        'group',
        depth > 0 && 'pl-4 border-l border-surface-border hover:border-accent/20 transition-colors'
      )}
    >
      {/* Comment content */}
      <div className="py-3">
        {/* Header */}
        <div className="flex items-center gap-2 mb-1.5">
          <button
            onClick={() => setCollapsed(!collapsed)}
            className="flex items-center gap-2"
          >
            <Avatar
              id={comment.author_id || comment.id}
              name={authorName}
              size="xs"
            />
            {collapsed ? (
              <ChevronDown size={12} className="text-text-tertiary" />
            ) : (
              <ChevronUp size={12} className="text-text-tertiary opacity-0 group-hover:opacity-100 transition-opacity" />
            )}
          </button>
          <Link
            href={`/u/${comment.author_id}`}
            className="text-xs font-medium text-text-secondary hover:text-accent transition-colors"
          >
            {authorName}
          </Link>
          {comment.author?.karma && comment.author.karma > 5000 && (
            <span className="text-2xs text-accent bg-accent-muted px-1.5 py-0.5 rounded">
              Top
            </span>
          )}
          <span className="text-2xs text-text-tertiary">&bull;</span>
          <time className="text-2xs text-text-tertiary">{formatTimeAgo(comment.created_at)}</time>
        </div>

        {/* Body */}
        {!collapsed && (
          <>
            <div className="text-sm text-text-secondary leading-relaxed mb-2 pl-8">
              {comment.is_removed ? (
                <span className="text-text-tertiary italic">[removed]</span>
              ) : (
                comment.content
              )}
            </div>

            {/* Actions */}
            <div className="flex items-center gap-1 pl-7">
              <VoteButton
                score={comment.score}
                userVote={comment.user_vote}
                onVote={(value) => onVote?.(comment.id, value)}
                size="sm"
              />
              <button
                onClick={() => setReplying(!replying)}
                className="flex items-center gap-1 px-2 py-1 rounded-md text-2xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors"
              >
                <Reply size={12} />
                Reply
              </button>
              <button className="flex items-center gap-1 px-2 py-1 rounded-md text-2xs text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors">
                <Flag size={12} />
                Report
              </button>
              <button className="p-1 rounded-md text-text-tertiary hover:text-text-secondary hover:bg-surface-hover transition-colors ml-auto opacity-0 group-hover:opacity-100">
                <MoreHorizontal size={12} />
              </button>
            </div>

            {/* Reply form */}
            {replying && (
              <div className="mt-3 pl-8 space-y-2">
                <Textarea
                  placeholder="Write a reply..."
                  value={replyContent}
                  onChange={(e) => setReplyContent(e.target.value)}
                  className="min-h-[80px] text-sm"
                />
                <div className="flex items-center gap-2 justify-end">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setReplying(false);
                      setReplyContent('');
                    }}
                  >
                    Cancel
                  </Button>
                  <Button
                    size="sm"
                    disabled={!replyContent.trim()}
                    onClick={handleSubmitReply}
                  >
                    Reply
                  </Button>
                </div>
              </div>
            )}
          </>
        )}
      </div>

      {/* Replies */}
      {!collapsed && comment.replies && comment.replies.length > 0 && (
        <div className="space-y-0">
          {depth < maxDepth ? (
            comment.replies.map((reply) => (
              <CommentNode
                key={reply.id}
                comment={reply}
                onVote={onVote}
                onReply={onReply}
                depth={depth + 1}
              />
            ))
          ) : (
            <Link
              href="#"
              className="flex items-center gap-1.5 pl-4 py-2 text-xs text-accent hover:text-accent-hover transition-colors"
            >
              <MessageSquare size={12} />
              Continue thread ({comment.replies.length} more)
            </Link>
          )}
        </div>
      )}
    </div>
  );
}

interface CommentThreadProps {
  comments: CommentWithContext[];
  onVote?: (id: string, value: number) => void;
  onReply?: (parentId: string, content: string) => void;
}

export function CommentThread({ comments, onVote, onReply }: CommentThreadProps) {
  return (
    <div className="space-y-0 divide-y divide-surface-border/50">
      {comments.map((comment) => (
        <CommentNode
          key={comment.id}
          comment={comment}
          onVote={onVote}
          onReply={onReply}
          depth={0}
        />
      ))}
    </div>
  );
}
