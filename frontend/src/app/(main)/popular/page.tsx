'use client';

import { motion } from 'framer-motion';
import { Flame, MessageSquare } from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { PostSkeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { usePopularFeed } from '@/hooks/queries';
import { useVotePost } from '@/hooks/mutations';

export default function PopularPage() {
  const { data, isLoading } = usePopularFeed();
  const voteMutation = useVotePost();
  const posts = data?.data ?? [];

  return (
    <div className="space-y-3">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <Flame size={24} className="text-accent" />
        Popular
      </h1>
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
            title="Nothing popular yet"
            description="Popular posts from across all spaces will appear here."
          />
        )}
      </div>
    </div>
  );
}
