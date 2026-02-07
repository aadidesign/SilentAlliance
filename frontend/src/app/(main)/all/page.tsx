'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Globe, Flame, Clock, Award, TrendingUp } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { Tabs } from '@/components/ui/Tabs';
import type { PostWithContext, PostSort } from '@/types';

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

const allPosts: PostWithContext[] = [
  {
    id: 'all1', space_id: 's3', author_id: 'a3',
    title: 'How I anonymized my entire digital footprint in 30 days',
    content: 'Step-by-step guide covering email, browsing, payments, and more...',
    content_type: 'text', url: null, media_ids: [],
    upvotes: 456, downvotes: 12, score: 444, comment_count: 123,
    is_pinned: false, is_locked: false, is_removed: false, removed_reason: null,
    created_at: new Date(Date.now() - 1800000).toISOString(),
    updated_at: new Date(Date.now() - 1800000).toISOString(),
    author: { id: 'a3', public_key_fingerprint: 'def', display_name: 'DigitalNomad', avatar_hash: null, bio: null, karma: 6700, is_verified: false, created_at: '' },
    space: { id: 's3', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8200 },
    user_vote: null,
  },
  {
    id: 'all2', space_id: 's4', author_id: 'a4',
    title: 'Decentralized social networks: The next frontier',
    content: 'Why ActivityPub and Nostr may define the future of social media...',
    content_type: 'text', url: null, media_ids: [],
    upvotes: 321, downvotes: 45, score: 276, comment_count: 89,
    is_pinned: false, is_locked: false, is_removed: false, removed_reason: null,
    created_at: new Date(Date.now() - 5400000).toISOString(),
    updated_at: new Date(Date.now() - 5400000).toISOString(),
    author: { id: 'a4', public_key_fingerprint: 'ghi', display_name: null, avatar_hash: null, bio: null, karma: 1200, is_verified: false, created_at: '' },
    space: { id: 's4', name: 'crypto', slug: 'crypto', icon_url: null, subscriber_count: 12400 },
    user_vote: null,
  },
];

export default function AllPage() {
  const [activeSort, setActiveSort] = useState<PostSort>('hot');

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <Globe size={24} className="text-accent" />
        All
      </h1>
      <Tabs tabs={sortTabs} activeTab={activeSort} onChange={(id) => setActiveSort(id as PostSort)} />
      <div className="space-y-3">
        {allPosts.map((post, i) => (
          <motion.div
            key={post.id}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: i * 0.05 }}
          >
            <PostCard post={post} onVote={(value) => console.log('Vote:', value)} />
          </motion.div>
        ))}
      </div>
    </div>
  );
}
