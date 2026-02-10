'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  Users,
  FileText,
  Clock,
  Shield,
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
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Tabs } from '@/components/ui/Tabs';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatNumber, formatTimeAgo } from '@/lib/utils';
import { useAuthStore } from '@/lib/store';
import type { PostWithContext, PostSort, Space } from '@/types';

// Sample space data
const sampleSpace: Space = {
  id: 'sp1',
  name: 'privacy',
  slug: 'privacy',
  description:
    'A community dedicated to discussing online privacy, surveillance, encryption, and digital rights. Share news, tools, and strategies for protecting your digital life.',
  rules: ['Be respectful', 'No doxxing', 'Cite sources', 'No illegal content'],
  icon_url: null,
  banner_url: null,
  is_private: false,
  is_nsfw: false,
  creator_id: 'c1',
  subscriber_count: 8234,
  post_count: 1247,
  created_at: new Date(Date.now() - 86400000 * 180).toISOString(),
  updated_at: new Date(Date.now() - 3600000).toISOString(),
};

const sampleSpacePosts: PostWithContext[] = [
  {
    id: 'sp1',
    space_id: 'sp1',
    author_id: 'a1',
    title: 'VPN comparison for 2026: Which services actually respect your privacy?',
    content: 'I\'ve spent the past month auditing the top VPN services. Here are my findings based on their actual data practices, not marketing claims...',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 567,
    downvotes: 23,
    score: 544,
    comment_count: 198,
    is_pinned: true,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 4).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 4).toISOString(),
    author: { id: 'a1', public_key_fingerprint: 'abc123', display_name: 'SecurityResearcher', avatar_hash: null, bio: null, karma: 7800, is_verified: true, created_at: '' },
    space: { id: 'sp1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8234 },
    user_vote: 1,
  },
  {
    id: 'sp2',
    space_id: 'sp1',
    author_id: 'a2',
    title: 'I built a browser extension that blocks invisible tracking pixels',
    content: 'After discovering how many emails contain tracking pixels, I decided to build a solution...',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 312,
    downvotes: 8,
    score: 304,
    comment_count: 76,
    is_pinned: false,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 3600000 * 9).toISOString(),
    updated_at: new Date(Date.now() - 3600000 * 9).toISOString(),
    author: { id: 'a2', public_key_fingerprint: 'def456', display_name: 'PixelBlocker', avatar_hash: null, bio: null, karma: 3200, is_verified: false, created_at: '' },
    space: { id: 'sp1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8234 },
    user_vote: null,
  },
];

const sortTabs = [
  { id: 'hot' as PostSort, label: 'Hot', icon: <Flame size={14} /> },
  { id: 'new' as PostSort, label: 'New', icon: <Clock size={14} /> },
  { id: 'top' as PostSort, label: 'Top', icon: <Award size={14} /> },
  { id: 'rising' as PostSort, label: 'Rising', icon: <TrendingUp size={14} /> },
];

export default function SpacePage() {
  const params = useParams();
  const router = useRouter();
  const { isAuthenticated } = useAuthStore();
  const [activeSort, setActiveSort] = useState<PostSort>('hot');
  const [joined, setJoined] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const space = sampleSpace;
  const posts = sampleSpacePosts;

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
                      onClick={() => setJoined(!joined)}
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
      {(space.description || space.rules.length > 0) && (
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
              {space.rules.length > 0 && (
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
          onClick={() => router.push(`/s/${params.slug}/submit`)}
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
        {posts.map((post, i) => (
          <motion.div
            key={post.id}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
          >
            <PostCard
              post={post}
              onVote={(value) => console.log('Vote:', post.id, value)}
            />
          </motion.div>
        ))}
      </div>
    </div>
  );
}
