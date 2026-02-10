'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Flame, Clock, TrendingUp, Award, Zap, PenLine, MessageSquare } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { PostSkeleton } from '@/components/ui/Skeleton';
import { Tabs } from '@/components/ui/Tabs';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { Card } from '@/components/ui/Card';
import { useAuthStore } from '@/lib/store';
import type { PostWithContext, PostSort } from '@/types';

// Sample data for demonstration
const samplePosts: PostWithContext[] = [
  {
    id: '1',
    space_id: 's1',
    author_id: 'a1',
    title: 'The state of privacy in 2026: Are we losing the battle?',
    content:
      'With recent developments in surveillance technology and data collection, it feels like personal privacy is becoming more of a luxury than a right. What are your thoughts on the current landscape and what we can do to protect ourselves?',
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
    author: { id: 'a1', public_key_fingerprint: 'abc123def', display_name: 'PrivacyAdvocate', avatar_hash: null, bio: null, karma: 4200, is_verified: true, created_at: '' },
    space: { id: 's1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8200 },
    user_vote: null,
  },
  {
    id: '2',
    space_id: 's2',
    author_id: 'a2',
    title: 'Introducing zero-knowledge proofs for anonymous voting systems',
    content:
      'I\'ve been working on a ZK-proof based voting system that allows for verifiable, anonymous votes. Here\'s a technical deep-dive into the approach...',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 521,
    downvotes: 12,
    score: 509,
    comment_count: 143,
    is_pinned: true,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 5).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 5).toISOString(),
    author: { id: 'a2', public_key_fingerprint: 'xyz789abc', display_name: 'CryptoEngineer', avatar_hash: null, bio: null, karma: 12300, is_verified: true, created_at: '' },
    space: { id: 's2', name: 'crypto', slug: 'crypto', icon_url: null, subscriber_count: 12400 },
    user_vote: 1,
  },
  {
    id: '3',
    space_id: 's3',
    author_id: 'a3',
    title: 'How DeFi protocols are enabling censorship-resistant finance',
    content: null,
    content_type: 'link',
    url: 'https://example.com/defi-censorship-resistance',
    media_ids: [],
    upvotes: 198,
    downvotes: 45,
    score: 153,
    comment_count: 62,
    is_pinned: false,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 8).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 8).toISOString(),
    author: { id: 'a3', public_key_fingerprint: 'def456ghi', display_name: null, avatar_hash: null, bio: null, karma: 890, is_verified: false, created_at: '' },
    space: { id: 's3', name: 'defi', slug: 'defi', icon_url: null, subscriber_count: 9800 },
    user_vote: null,
  },
  {
    id: '4',
    space_id: 's4',
    author_id: 'a4',
    title: 'A simple guide to setting up your own Tor hidden service',
    content:
      'This guide walks you through setting up a Tor hidden service for hosting a simple website that is accessible only through the Tor network. This can be useful for whistleblowing platforms, anonymous blogs, and more.',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 87,
    downvotes: 5,
    score: 82,
    comment_count: 29,
    is_pinned: false,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 12).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 12).toISOString(),
    author: { id: 'a4', public_key_fingerprint: 'ghi789jkl', display_name: 'TorExpert', avatar_hash: null, bio: null, karma: 2100, is_verified: false, created_at: '' },
    space: { id: 's4', name: 'technology', slug: 'technology', icon_url: null, subscriber_count: 34100 },
    user_vote: -1,
  },
  {
    id: '5',
    space_id: 's1',
    author_id: 'a5',
    title: 'EU\'s new digital identity regulation: What it means for online anonymity',
    content:
      'The European Union has proposed new regulations around digital identity verification. Let\'s discuss the implications for platforms that prioritize user anonymity.',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 256,
    downvotes: 34,
    score: 222,
    comment_count: 98,
    is_pinned: false,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 18).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 18).toISOString(),
    author: { id: 'a5', public_key_fingerprint: 'mno012pqr', display_name: 'PolicyWatcher', avatar_hash: null, bio: null, karma: 5600, is_verified: true, created_at: '' },
    space: { id: 's1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8200 },
    user_vote: null,
  },
];

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

export default function FeedPage() {
  const router = useRouter();
  const { isAuthenticated } = useAuthStore();
  const [activeSort, setActiveSort] = useState<PostSort>('hot');
  // TODO: Replace with React Query hook — const { data: posts, isLoading } = useFeedPosts(activeSort);
  const [posts] = useState<PostWithContext[]>(samplePosts);
  const isLoading = false;

  return (
    <div className="space-y-3">
      {/* Create post prompt */}
      {isAuthenticated && (
        <Card
          padding="sm"
          hover
          onClick={() => router.push('/submit')}
          className="flex items-center gap-3 px-4"
        >
          <div className="w-9 h-9 rounded-full bg-bg-tertiary flex items-center justify-center shrink-0">
            <PenLine size={16} className="text-text-tertiary" />
          </div>
          <div className="flex-1 h-9 bg-bg-tertiary rounded-lg flex items-center px-3">
            <span className="text-sm text-text-tertiary">Create a post...</span>
          </div>
        </Card>
      )}

      {/* Sort tabs */}
      <Tabs
        tabs={sortTabs}
        activeTab={activeSort}
        onChange={(id) => setActiveSort(id as PostSort)}
      />

      {/* Posts */}
      <div className="space-y-3">
        {isLoading ? (
          <>
            {Array.from({ length: 4 }).map((_, i) => (
              <PostSkeleton key={i} />
            ))}
          </>
        ) : posts.length > 0 ? (
          posts.map((post, i) => (
            <motion.div
              key={post.id}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
            >
              <PostCard
                post={post}
                onVote={(value) => {
                  console.log('Vote:', post.id, value);
                }}
              />
            </motion.div>
          ))
        ) : (
          <EmptyState
            icon={<MessageSquare size={28} />}
            title="No posts yet"
            description="Be the first to start a conversation in this community."
            action={{
              label: 'Create Post',
              onClick: () => router.push('/submit'),
            }}
          />
        )}
      </div>

      {/* Load more */}
      {posts.length > 0 && (
        <div className="flex justify-center py-4">
          <Button variant="ghost" size="sm">
            Load more
          </Button>
        </div>
      )}
    </div>
  );
}
