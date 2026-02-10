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
  MoreHorizontal,
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
import { CommentThread } from '@/components/comment/CommentThread';
import { cn, formatTimeAgo, formatNumber } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { useAuthStore } from '@/lib/store';
import type { PostWithContext, CommentWithContext } from '@/types';

// Sample data
const samplePost: PostWithContext = {
  id: 'p1',
  space_id: 'sp1',
  author_id: 'a1',
  title: 'The state of privacy in 2026: Are we losing the battle?',
  content: `With recent developments in surveillance technology and data collection, it feels like personal privacy is becoming more of a luxury than a right.

## The Current Landscape

Over the past year, we've seen:

- **Government surveillance expansion**: Multiple countries have passed laws mandating backdoors in encryption
- **Corporate data harvesting**: Big tech companies have been caught collecting more data than ever
- **AI-powered tracking**: New AI models can identify individuals across multiple data sources

## What Can We Do?

1. **Use end-to-end encryption** for all communications
2. **Self-host** critical services where possible
3. **Support organizations** fighting for digital rights
4. **Educate others** about privacy tools and practices

The battle isn't lost, but it requires collective action. What are your thoughts? What privacy tools and practices do you recommend?`,
  content_type: 'text',
  url: null,
  media_ids: [],
  upvotes: 342,
  downvotes: 18,
  score: 324,
  comment_count: 87,
  is_pinned: false,
  is_locked: false,
  is_removed: false,
  removed_reason: null,
  created_at: new Date(Date.now() - 3600000 * 2).toISOString(),
  updated_at: new Date(Date.now() - 3600000 * 2).toISOString(),
  author: {
    id: 'a1',
    public_key_fingerprint: 'abc123def456',
    display_name: 'PrivacyAdvocate',
    avatar_hash: null,
    bio: null,
    karma: 4200,
    is_verified: true,
    created_at: '',
  },
  space: {
    id: 'sp1',
    name: 'privacy',
    slug: 'privacy',
    icon_url: null,
    subscriber_count: 8200,
  },
  user_vote: null,
};

const sampleComments: CommentWithContext[] = [
  {
    id: 'c1',
    post_id: 'p1',
    parent_id: null,
    author_id: 'a2',
    content: 'Great analysis. I think the most impactful thing we can do is normalize the use of encrypted communications. When only "suspicious" people use encryption, it becomes a flag. When everyone does, it\'s just standard practice.',
    depth: 0,
    path: 'c1',
    upvotes: 156,
    downvotes: 3,
    score: 153,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 1.5).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 1.5).toISOString(),
    author: {
      id: 'a2',
      public_key_fingerprint: 'xyz789',
      display_name: 'CryptoEnthusiast',
      avatar_hash: null,
      bio: null,
      karma: 8900,
      is_verified: true,
      created_at: '',
    },
    user_vote: 1,
    replies: [
      {
        id: 'c3',
        post_id: 'p1',
        parent_id: 'c1',
        author_id: 'a4',
        content: 'Exactly this. Signal adoption has been growing but we need it to be the default, not the exception.',
        depth: 1,
        path: 'c1/c3',
        upvotes: 67,
        downvotes: 1,
        score: 66,
        is_removed: false,
        removed_reason: null,
        created_at: new Date(Date.now() - 3600000 * 1).toISOString(),
        updated_at: new Date(Date.now() - 3600000 * 1).toISOString(),
        author: {
          id: 'a4',
          public_key_fingerprint: 'def456',
          display_name: null,
          avatar_hash: null,
          bio: null,
          karma: 1200,
          is_verified: false,
          created_at: '',
        },
        user_vote: null,
        replies: [],
      },
    ],
  },
  {
    id: 'c2',
    post_id: 'p1',
    parent_id: null,
    author_id: 'a3',
    content: 'I\'d add that we should also be supporting open-source alternatives to popular services. Every time you use a FOSS alternative, you\'re voting with your usage for a more private future.',
    depth: 0,
    path: 'c2',
    upvotes: 89,
    downvotes: 5,
    score: 84,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 1.2).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 1.2).toISOString(),
    author: {
      id: 'a3',
      public_key_fingerprint: 'ghi012',
      display_name: 'FOSSAdvocate',
      avatar_hash: null,
      bio: null,
      karma: 3400,
      is_verified: false,
      created_at: '',
    },
    user_vote: null,
    replies: [
      {
        id: 'c4',
        post_id: 'p1',
        parent_id: 'c2',
        author_id: 'a1',
        content: 'Agreed. I recently switched to Nextcloud for file storage and it\'s been a game changer. Self-hosting gives you complete control.',
        depth: 1,
        path: 'c2/c4',
        upvotes: 45,
        downvotes: 0,
        score: 45,
        is_removed: false,
        removed_reason: null,
        created_at: new Date(Date.now() - 3600000 * 0.8).toISOString(),
        updated_at: new Date(Date.now() - 3600000 * 0.8).toISOString(),
        author: {
          id: 'a1',
          public_key_fingerprint: 'abc123def456',
          display_name: 'PrivacyAdvocate',
          avatar_hash: null,
          bio: null,
          karma: 4200,
          is_verified: true,
          created_at: '',
        },
        user_vote: null,
        replies: [],
      },
    ],
  },
];

const commentSortTabs = [
  { id: 'best', label: 'Best', icon: <Award size={14} /> },
  { id: 'new', label: 'New', icon: <Clock size={14} /> },
];

export default function PostDetailPage() {
  const params = useParams();
  const router = useRouter();
  const { isAuthenticated } = useAuthStore();
  const [post] = useState(samplePost);
  const [comments] = useState(sampleComments);
  const [commentSort, setCommentSort] = useState('best');
  const [newComment, setNewComment] = useState('');

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
              onVote={(value) => console.log('Vote:', value)}
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
              disabled={!newComment.trim()}
              onClick={() => {
                console.log('Submit comment:', newComment);
                setNewComment('');
              }}
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
          {comments.length > 0 ? (
            <CommentThread
              comments={comments}
              onVote={(id, value) => console.log('Comment vote:', id, value)}
              onReply={(parentId, content) =>
                console.log('Reply to:', parentId, content)
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
