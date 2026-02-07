'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Flame, Clock, Award, TrendingUp } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { Tabs } from '@/components/ui/Tabs';
import type { PostWithContext, PostSort } from '@/types';

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

// Reuse sample data structure
const popularPosts: PostWithContext[] = [
  {
    id: 'pop1', space_id: 's1', author_id: 'a1',
    title: 'Breaking: Major tech company caught selling encrypted user data to advertisers',
    content: 'Documents leaked today reveal a systematic process...',
    content_type: 'text', url: null, media_ids: [],
    upvotes: 2341, downvotes: 89, score: 2252, comment_count: 567,
    is_pinned: false, is_locked: false, is_removed: false, removed_reason: null,
    created_at: new Date(Date.now() - 3600000).toISOString(),
    updated_at: new Date(Date.now() - 3600000).toISOString(),
    author: { id: 'a1', public_key_fingerprint: 'abc', display_name: 'Whistleblower42', avatar_hash: null, bio: null, karma: 15000, is_verified: true, created_at: '' },
    space: { id: 's1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8200 },
    user_vote: null,
  },
  {
    id: 'pop2', space_id: 's2', author_id: 'a2',
    title: 'New Rust zero-day vulnerability patched in record time by open-source community',
    content: null, content_type: 'link', url: 'https://example.com/rust-zeroday', media_ids: [],
    upvotes: 1876, downvotes: 34, score: 1842, comment_count: 342,
    is_pinned: false, is_locked: false, is_removed: false, removed_reason: null,
    created_at: new Date(Date.now() - 7200000).toISOString(),
    updated_at: new Date(Date.now() - 7200000).toISOString(),
    author: { id: 'a2', public_key_fingerprint: 'xyz', display_name: 'RustDev', avatar_hash: null, bio: null, karma: 9800, is_verified: false, created_at: '' },
    space: { id: 's2', name: 'technology', slug: 'technology', icon_url: null, subscriber_count: 34100 },
    user_vote: 1,
  },
];

export default function PopularPage() {
  const [activeSort, setActiveSort] = useState<PostSort>('hot');

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <Flame size={24} className="text-accent" />
        Popular
      </h1>
      <Tabs tabs={sortTabs} activeTab={activeSort} onChange={(id) => setActiveSort(id as PostSort)} />
      <div className="space-y-3">
        {popularPosts.map((post, i) => (
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
