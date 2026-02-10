'use client';

import { motion } from 'framer-motion';
import { TrendingUp, Users, ArrowUp } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { EmptyState } from '@/components/ui/EmptyState';
import { Skeleton } from '@/components/ui/Skeleton';
import Link from 'next/link';
import { formatNumber } from '@/lib/utils';
import { useSpaces, usePopularFeed } from '@/hooks/queries';

export default function TrendingPage() {
  const { data: spacesData, isLoading: spacesLoading } = useSpaces();
  const { data: popularData, isLoading: postsLoading } = usePopularFeed();

  // Sort spaces by subscriber_count descending as proxy for "trending"
  const trendingSpaces = [...(spacesData?.data ?? [])].sort(
    (a, b) => b.subscriber_count - a.subscriber_count
  ).slice(0, 10);

  const popularPosts = popularData?.data ?? [];

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <TrendingUp size={24} className="text-accent" />
        Trending
      </h1>

      {/* Trending Spaces */}
      <div>
        <h2 className="text-sm font-semibold text-text-secondary mb-3 uppercase tracking-wider">
          Top Spaces
        </h2>
        <div className="space-y-2">
          {spacesLoading ? (
            Array.from({ length: 5 }).map((_, i) => (
              <Card key={i} padding="sm" className="flex items-center gap-4 px-4">
                <Skeleton className="w-6 h-5" />
                <Skeleton className="w-10 h-10 rounded-xl" />
                <div className="flex-1 space-y-1">
                  <Skeleton className="w-24 h-4" />
                  <Skeleton className="w-36 h-3" />
                </div>
              </Card>
            ))
          ) : trendingSpaces.length === 0 ? (
            <EmptyState
              icon={<TrendingUp size={28} />}
              title="No spaces yet"
              description="Create the first space and start building a community."
            />
          ) : (
            trendingSpaces.map((space, i) => (
              <motion.div
                key={space.id}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
              >
                <Link href={`/s/${space.slug}`}>
                  <Card hover padding="sm" className="flex items-center gap-4 px-4">
                    <span className="text-lg font-bold text-text-tertiary w-6 text-right">
                      {i + 1}
                    </span>
                    <div className="w-10 h-10 rounded-xl bg-surface-hover flex items-center justify-center shrink-0">
                      <span className="text-sm font-bold text-accent">
                        {space.name.charAt(0).toUpperCase()}
                      </span>
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-text-primary">s/{space.slug}</p>
                      <p className="text-xs text-text-tertiary">
                        {formatNumber(space.subscriber_count)} members &bull; {formatNumber(space.post_count)} posts
                      </p>
                    </div>
                    <Badge variant="success">
                      <Users size={10} className="mr-0.5" />
                      {formatNumber(space.subscriber_count)}
                    </Badge>
                  </Card>
                </Link>
              </motion.div>
            ))
          )}
        </div>
      </div>

      {/* Popular Posts */}
      <div>
        <h2 className="text-sm font-semibold text-text-secondary mb-3 uppercase tracking-wider">
          Popular Right Now
        </h2>
        <div className="space-y-2">
          {postsLoading ? (
            Array.from({ length: 3 }).map((_, i) => (
              <Card key={i} padding="sm" className="space-y-2 px-4">
                <Skeleton className="w-3/4 h-4" />
                <Skeleton className="w-1/2 h-3" />
              </Card>
            ))
          ) : popularPosts.length === 0 ? (
            <Card className="text-center py-8 text-sm text-text-tertiary">
              No popular posts yet.
            </Card>
          ) : (
            popularPosts.slice(0, 10).map((post, i) => (
              <motion.div
                key={post.id}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2, delay: Math.min(i * 0.03, 0.3) }}
              >
                <Link href={`/post/${post.id}`}>
                  <Card hover padding="sm" className="px-4">
                    <p className="text-sm font-medium text-text-primary line-clamp-1">
                      {post.title}
                    </p>
                    <p className="text-xs text-text-tertiary mt-0.5">
                      s/{post.space?.slug} &bull; {formatNumber(post.score)} points &bull; {formatNumber(post.comment_count)} comments
                    </p>
                  </Card>
                </Link>
              </motion.div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
