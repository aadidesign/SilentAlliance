'use client';

import { useState } from 'react';
import { useParams } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  Shield,
  Calendar,
  Star,
  MessageSquare,
  FileText,
  Copy,
  Check,
  ExternalLink,
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Tabs } from '@/components/ui/Tabs';
import { PostCard } from '@/components/post/PostCard';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatNumber, formatTimeAgo, shortenFingerprint } from '@/lib/utils';
import type { IdentityPublic, PostWithContext } from '@/types';

const sampleIdentity: IdentityPublic = {
  id: 'a1',
  public_key_fingerprint: 'abc123def456789abcdef0123456789abcdef0123456789abcdef0123456789a',
  display_name: 'PrivacyAdvocate',
  avatar_hash: null,
  bio: 'Fighting for digital rights and online privacy. Crypto enthusiast. Open-source contributor.',
  karma: 4200,
  is_verified: true,
  created_at: new Date(Date.now() - 86400000 * 365).toISOString(),
};

const sampleUserPosts: PostWithContext[] = [
  {
    id: 'up1',
    space_id: 'sp1',
    author_id: 'a1',
    title: 'The state of privacy in 2026: Are we losing the battle?',
    content: 'With recent developments in surveillance technology...',
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
    author: sampleIdentity,
    space: { id: 'sp1', name: 'privacy', slug: 'privacy', icon_url: null, subscriber_count: 8200 },
    user_vote: null,
  },
  {
    id: 'up2',
    space_id: 'sp2',
    author_id: 'a1',
    title: 'Self-hosting guide: Take back control of your data',
    content: 'A comprehensive guide to self-hosting essential services...',
    content_type: 'text',
    url: null,
    media_ids: [],
    upvotes: 189,
    downvotes: 7,
    score: 182,
    comment_count: 45,
    is_pinned: false,
    is_locked: false,
    is_removed: false,
    removed_reason: null,
    created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
    updated_at: new Date(Date.now() - 86400000 * 3).toISOString(),
    author: sampleIdentity,
    space: { id: 'sp2', name: 'technology', slug: 'technology', icon_url: null, subscriber_count: 34100 },
    user_vote: 1,
  },
];

const profileTabs = [
  { id: 'posts', label: 'Posts', icon: <FileText size={14} /> },
  { id: 'comments', label: 'Comments', icon: <MessageSquare size={14} /> },
];

export default function ProfilePage() {
  const params = useParams();
  const [activeTab, setActiveTab] = useState('posts');
  const [copied, setCopied] = useState(false);
  const identity = sampleIdentity;

  const handleCopyFingerprint = () => {
    navigator.clipboard.writeText(identity.public_key_fingerprint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-6">
      {/* Profile header */}
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.3 }}
      >
        <Card className="p-6">
          <div className="flex flex-col sm:flex-row items-start gap-5">
            <Avatar id={identity.id} name={identity.display_name} size="xl" />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-1">
                <h1 className="text-xl font-bold text-text-primary">
                  {identity.display_name || 'Anonymous'}
                </h1>
                {identity.is_verified && (
                  <Badge variant="accent">
                    <Shield size={10} className="mr-1" /> Verified
                  </Badge>
                )}
              </div>

              {/* Fingerprint */}
              <button
                onClick={handleCopyFingerprint}
                className="flex items-center gap-1.5 text-xs font-mono text-text-tertiary hover:text-text-secondary transition-colors mb-3"
              >
                {shortenFingerprint(identity.public_key_fingerprint)}
                {copied ? <Check size={12} className="text-success" /> : <Copy size={12} />}
              </button>

              {/* Bio */}
              {identity.bio && (
                <p className="text-sm text-text-secondary leading-relaxed mb-4">
                  {identity.bio}
                </p>
              )}

              {/* Stats */}
              <div className="flex items-center gap-6 text-sm">
                <div className="flex items-center gap-1.5">
                  <Star size={14} className="text-accent" />
                  <span className="font-medium text-text-primary">
                    {formatNumber(identity.karma)}
                  </span>
                  <span className="text-text-tertiary">karma</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <Calendar size={14} className="text-text-tertiary" />
                  <span className="text-text-tertiary">
                    Joined {formatTimeAgo(identity.created_at)}
                  </span>
                </div>
              </div>
            </div>

            {/* Actions */}
            <div className="flex items-center gap-2 shrink-0">
              <Button variant="primary" size="sm" leftIcon={<MessageSquare size={14} />}>
                Message
              </Button>
            </div>
          </div>
        </Card>
      </motion.div>

      {/* Tabs */}
      <Tabs tabs={profileTabs} activeTab={activeTab} onChange={setActiveTab} />

      {/* Content */}
      <div className="space-y-3">
        {activeTab === 'posts' && (
          <>
            {sampleUserPosts.map((post, i) => (
              <motion.div
                key={post.id}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3, delay: i * 0.05 }}
              >
                <PostCard post={post} onVote={(value) => console.log('Vote:', value)} />
              </motion.div>
            ))}
          </>
        )}

        {activeTab === 'comments' && (
          <EmptyState
            icon={<MessageSquare size={28} />}
            title="No comments yet"
            description="This user hasn't commented on anything yet."
          />
        )}
      </div>
    </div>
  );
}
