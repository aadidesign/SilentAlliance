'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  Users,
  FileText,
  Clock,
  Settings,
  Plus,
  Flame,
  Award,
  TrendingUp,
  Lock,
  Globe,
  ChevronDown,
} from 'lucide-react';
import { PostCard } from '@/components/post/PostCard';
import { PostSkeleton } from '@/components/ui/Skeleton';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Tabs } from '@/components/ui/Tabs';
import { Card } from '@/components/ui/Card';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatNumber } from '@/lib/utils';
import { useAuthStore } from '@/lib/store';
import { useSpace, useSpacePosts } from '@/hooks/queries';
import { useVotePost, useJoinSpace, useLeaveSpace } from '@/hooks/mutations';
import type { PostSort } from '@/types';

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

export default function SpacePage() {
  const params = useParams();
  const router = useRouter();
  const slug = params.slug as string;
  const { isAuthenticated } = useAuthStore();
  const [activeSort, setActiveSort] = useState<PostSort>('hot');
  const [showAbout, setShowAbout] = useState(false);

  const { data: space, isLoading: spaceLoading } = useSpace(slug);
  const { data: postsData, isLoading: postsLoading } = useSpacePosts(slug, activeSort);
  const voteMutation = useVotePost();
  const joinMutation = useJoinSpace();
  const leaveMutation = useLeaveSpace();

  const posts = postsData?.data ?? [];

  // Simple joined state tracking (would come from space membership API in production)
  const [joined, setJoined] = useState(false);

  const handleJoinLeave = () => {
    if (joined) {
      leaveMutation.mutate(slug, { onSuccess: () => setJoined(false) });
    } else {
      joinMutation.mutate(slug, { onSuccess: () => setJoined(true) });
    }
  };

  if (spaceLoading) {
    return (
      <div className="space-y-4">
        <div className="h-32 sm:h-40 -mx-4 -mt-6 bg-gradient-to-r from-accent/20 via-accent-secondary/10 to-accent/20" />
        <div className="flex items-start gap-4 -mt-8 relative">
          <Skeleton className="w-16 h-16 rounded-2xl" />
          <div className="flex-1 space-y-2">
            <Skeleton className="w-40 h-7" />
            <Skeleton className="w-60 h-4" />
          </div>
        </div>
        <div className="space-y-3 mt-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <PostSkeleton key={i} />
          ))}
        </div>
      </div>
    );
  }

  if (!space) {
    return (
      <div className="text-center py-20">
        <Globe size={32} className="mx-auto text-text-tertiary mb-3" />
        <h2 className="text-lg font-semibold text-text-primary mb-1">Space not found</h2>
        <p className="text-sm text-text-tertiary mb-4">This space may have been deleted or doesn&apos;t exist.</p>
        <Button variant="secondary" onClick={() => router.push('/all')}>
          Browse All Posts
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Banner */}
      <div className="relative h-32 sm:h-40 -mx-4 -mt-6 bg-gradient-to-r from-accent/20 via-accent-secondary/10 to-accent/20 overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-transparent to-bg-primary" />
      </div>

      {/* Space header */}
      <div className="-mt-8 relative">
        <div className="flex flex-col sm:flex-row items-start gap-4">
          {/* Icon */}
          <div className="w-16 h-16 rounded-2xl bg-surface border-4 border-bg-primary flex items-center justify-center shrink-0">
            <span className="text-2xl font-bold text-accent">
              {space.name.charAt(0).toUpperCase()}
            </span>
          </div>

          {/* Info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-start justify-between gap-3">
              <div>
                <h1 className="text-2xl font-bold text-text-primary">s/{space.name}</h1>
                <div className="flex items-center gap-3 mt-1 text-sm text-text-tertiary">
                  <span className="flex items-center gap-1">
                    <Users size={14} />
                    {formatNumber(space.subscriber_count)} members
                  </span>
                  <span className="flex items-center gap-1">
                    <FileText size={14} />
                    {formatNumber(space.post_count)} posts
                  </span>
                  {space.is_private ? (
                    <Badge variant="warning">
                      <Lock size={10} className="mr-1" /> Private
                    </Badge>
                  ) : (
                    <Badge>
                      <Globe size={10} className="mr-1" /> Public
                    </Badge>
                  )}
                </div>
              </div>

              <div className="flex items-center gap-2 shrink-0">
                {isAuthenticated && (
                  <>
                    <Button
                      variant={joined ? 'secondary' : 'primary'}
                      size="sm"
                      onClick={handleJoinLeave}
                      isLoading={joinMutation.isPending || leaveMutation.isPending}
                    >
                      {joined ? 'Joined' : 'Join'}
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8"
                    >
                      <Settings size={16} />
                    </Button>
                  </>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* About & Rules (collapsible) */}
      {(space.description || (space.rules && space.rules.length > 0)) && (
        <Card padding="none">
          <button
            onClick={() => setShowAbout(!showAbout)}
            className="flex items-center justify-between w-full px-4 py-3 text-sm font-medium text-text-secondary hover:text-text-primary transition-colors"
          >
            <span>About this space</span>
            <ChevronDown
              size={16}
              className={cn(
                'text-text-tertiary transition-transform duration-200',
                showAbout && 'rotate-180'
              )}
            />
          </button>
          {showAbout && (
            <div className="px-4 pb-4 space-y-4 border-t border-surface-border pt-3">
              {space.description && (
                <p className="text-sm text-text-secondary leading-relaxed">
                  {space.description}
                </p>
              )}
              {space.rules && space.rules.length > 0 && (
                <div>
                  <h4 className="text-xs font-semibold text-text-tertiary uppercase tracking-wider mb-2">
                    Rules
                  </h4>
                  <ol className="space-y-1.5">
                    {space.rules.map((rule, i) => (
                      <li key={i} className="flex items-start gap-2 text-sm">
                        <span className="text-accent font-mono text-xs mt-0.5 shrink-0">
                          {String(i + 1).padStart(2, '0')}
                        </span>
                        <span className="text-text-secondary">{rule}</span>
                      </li>
                    ))}
                  </ol>
                </div>
              )}
            </div>
          )}
        </Card>
      )}

      {/* Create post */}
      {isAuthenticated && joined && (
        <Button
          variant="outline"
          className="w-full justify-start gap-2"
          onClick={() => router.push(`/s/${slug}/submit`)}
          leftIcon={<Plus size={16} />}
        >
          Create a post in s/{space.name}
        </Button>
      )}

      {/* Sort tabs */}
      <Tabs
        tabs={sortTabs}
        activeTab={activeSort}
        onChange={(id) => setActiveSort(id as PostSort)}
      />

      {/* Posts */}
      <div className="space-y-3">
        {postsLoading ? (
          Array.from({ length: 3 }).map((_, i) => <PostSkeleton key={i} />)
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
            icon={<FileText size={28} />}
            title="No posts yet"
            description="Be the first to post in this space."
            action={
              isAuthenticated && joined
                ? { label: 'Create Post', onClick: () => router.push(`/s/${slug}/submit`) }
                : undefined
            }
          />
        )}
      </div>
    </div>
  );
}
