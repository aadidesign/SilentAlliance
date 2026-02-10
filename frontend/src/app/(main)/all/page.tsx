'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import { Globe, Flame, Clock, Award, TrendingUp, MessageSquare } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { PostSkeleton } from '@/components/ui/Skeleton';
import { Tabs } from '@/components/ui/Tabs';
import { EmptyState } from '@/components/ui/EmptyState';
import { useAllFeed } from '@/hooks/queries';
import { useVotePost } from '@/hooks/mutations';
import type { PostSort } from '@/types';

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

export default function AllPage() {
  const [activeSort, setActiveSort] = useState<PostSort>('hot');
  const { data, isLoading } = useAllFeed(activeSort);
  const voteMutation = useVotePost();
  const posts = data?.data ?? [];

  return (
    <div className="space-y-3">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <Globe size={24} className="text-accent" />
        All
      </h1>
      <Tabs tabs={sortTabs} activeTab={activeSort} onChange={(id) => setActiveSort(id as PostSort)} />
      <div className="space-y-3">
        {isLoading ? (
          Array.from({ length: 4 }).map((_, i) => <PostSkeleton key={i} />)
        ) : posts.length > 0 ? (
          posts.map((post, i) => (
            <motion.div
              key={post.id}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
            >
              <PostCard post={post} onVote={(value) => voteMutation.mutate({ id: post.id, value })} />
            </motion.div>
          ))
        ) : (
          <EmptyState
            icon={<MessageSquare size={28} />}
            title="No posts yet"
            description="Be the first to start a conversation. Posts from all spaces will appear here."
          />
        )}
      </div>
    </div>
  );
}
