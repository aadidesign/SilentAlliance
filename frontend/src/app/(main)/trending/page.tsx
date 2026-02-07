'use client';

import { motion } from 'framer-motion';
import { TrendingUp, Users, ArrowUp } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import Link from 'next/link';
import { cn, formatNumber } from '@/lib/utils';

const trendingSpaces = [
  { slug: 'privacy', name: 'privacy', growth: '+12%', members: 8234, posts_today: 47 },
  { slug: 'crypto', name: 'crypto', growth: '+8%', members: 12400, posts_today: 89 },
  { slug: 'whistleblowers', name: 'whistleblowers', growth: '+23%', members: 5700, posts_today: 15 },
  { slug: 'defi', name: 'defi', growth: '+6%', members: 9800, posts_today: 34 },
  { slug: 'technology', name: 'technology', growth: '+4%', members: 34100, posts_today: 156 },
];

const trendingTopics = [
  { tag: 'encryption', posts: 234, trend: 'up' },
  { tag: 'zero-knowledge', posts: 178, trend: 'up' },
  { tag: 'surveillance', posts: 156, trend: 'up' },
  { tag: 'open-source', posts: 134, trend: 'up' },
  { tag: 'vpn', posts: 98, trend: 'up' },
  { tag: 'tor', posts: 87, trend: 'up' },
  { tag: 'privacy-tools', posts: 76, trend: 'up' },
  { tag: 'decentralization', posts: 65, trend: 'up' },
];

export default function TrendingPage() {
  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-text-primary flex items-center gap-2">
        <TrendingUp size={24} className="text-accent" />
        Trending
      </h1>

      {/* Trending Spaces */}
      <div>
        <h2 className="text-sm font-semibold text-text-secondary mb-3 uppercase tracking-wider">
          Growing Spaces
        </h2>
        <div className="space-y-2">
          {trendingSpaces.map((space, i) => (
            <motion.div
              key={space.slug}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.3, delay: i * 0.05 }}
            >
              <Link href={`/s/${space.slug}`}>
                <Card hover padding="sm" className="flex items-center gap-4 px-4">
                  <span className="text-lg font-bold text-text-tertiary w-6 text-right">
                    {i + 1}
                  </span>
                  <div className="w-10 h-10 rounded-xl bg-surface-hover flex items-center justify-center shrink-0">
                    <span className="text-sm font-bold gradient-text">
                      {space.name.charAt(0).toUpperCase()}
                    </span>
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-text-primary">s/{space.name}</p>
                    <p className="text-xs text-text-tertiary">
                      {formatNumber(space.members)} members &bull; {space.posts_today} posts today
                    </p>
                  </div>
                  <Badge variant="success">
                    <ArrowUp size={10} className="mr-0.5" />
                    {space.growth}
                  </Badge>
                </Card>
              </Link>
            </motion.div>
          ))}
        </div>
      </div>

      {/* Trending Topics */}
      <div>
        <h2 className="text-sm font-semibold text-text-secondary mb-3 uppercase tracking-wider">
          Hot Topics
        </h2>
        <Card>
          <div className="flex flex-wrap gap-2">
            {trendingTopics.map((topic, i) => (
              <motion.div
                key={topic.tag}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ duration: 0.2, delay: i * 0.03 }}
              >
                <button className="flex items-center gap-2 px-3 py-2 rounded-xl bg-bg-tertiary border border-surface-border hover:border-accent/20 hover:bg-accent-muted/5 transition-all duration-200 group">
                  <span className="text-sm text-text-secondary group-hover:text-accent transition-colors">
                    #{topic.tag}
                  </span>
                  <span className="text-2xs text-text-tertiary">
                    {topic.posts}
                  </span>
                </button>
              </motion.div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
