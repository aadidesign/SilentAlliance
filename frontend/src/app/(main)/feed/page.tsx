'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Flame, Clock, TrendingUp, Award, PenLine, MessageSquare } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { PostSkeleton } from '@/components/ui/Skeleton';
import { Tabs } from '@/components/ui/Tabs';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { Card } from '@/components/ui/Card';
import { useFeed } from '@/hooks/queries';
import { useVotePost } from '@/hooks/mutations';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import type { PostSort } from '@/types';

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

export default function FeedPage() {
  const router = useRouter();
  const { isAuthenticated } = useRequireAuth();
  const [activeSort, setActiveSort] = useState<PostSort>('hot');
  const { data, isLoading } = useFeed(activeSort);
  const voteMutation = useVotePost();
  const posts = data?.data ?? [];

  if (!isAuthenticated) return null;

  return (
    <div className="space-y-3">
      {/* Create post prompt */}
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
                onVote={(value) => voteMutation.mutate({ id: post.id, value })}
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
