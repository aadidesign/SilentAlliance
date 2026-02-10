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
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Tabs } from '@/components/ui/Tabs';
import { PostCard } from '@/components/post/PostCard';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { formatNumber, formatTimeAgo, shortenFingerprint } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { useIdentity, useIdentityPosts, useIdentityComments } from '@/hooks/queries';
import { useVotePost } from '@/hooks/mutations';

const profileTabs = [
  { id: 'posts', label: 'Posts', icon: <FileText size={14} /> },
  { id: 'comments', label: 'Comments', icon: <MessageSquare size={14} /> },
];

export default function ProfilePage() {
  const params = useParams();
  const userId = params.id as string;
  const [activeTab, setActiveTab] = useState('posts');
  const [copied, setCopied] = useState(false);

  const { data: identity, isLoading: identityLoading } = useIdentity(userId);
  const { data: postsData, isLoading: postsLoading } = useIdentityPosts(userId);
  const { data: commentsData, isLoading: commentsLoading } = useIdentityComments(userId);
  const voteMutation = useVotePost();

  const posts = postsData?.data ?? [];
  const comments = commentsData?.data ?? [];

  const handleCopyFingerprint = () => {
    if (!identity) return;
    navigator.clipboard.writeText(identity.public_key_fingerprint);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  if (identityLoading) {
    return (
      <div className="space-y-6">
        <Card className="p-6">
          <div className="flex items-start gap-5">
            <Skeleton className="w-16 h-16 rounded-full" />
            <div className="flex-1 space-y-3">
              <Skeleton className="w-40 h-6" />
              <Skeleton className="w-60 h-3" />
              <Skeleton className="w-full h-12" />
              <Skeleton className="w-48 h-4" />
            </div>
          </div>
        </Card>
      </div>
    );
  }

  if (!identity) {
    return (
      <div className="text-center py-20">
        <Shield size={32} className="mx-auto text-text-tertiary mb-3" />
        <h2 className="text-lg font-semibold text-text-primary mb-1">User not found</h2>
        <p className="text-sm text-text-tertiary">This identity may not exist.</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Profile header */}
      <motion.div
        initial={pageEntrance.initial}
        animate={pageEntrance.animate}
        transition={pageEntrance.transition}
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
            {postsLoading ? (
              Array.from({ length: 2 }).map((_, i) => (
                <Card key={i} className="p-4 space-y-2">
                  <Skeleton className="w-3/4 h-5" />
                  <Skeleton className="w-1/2 h-3" />
                </Card>
              ))
            ) : posts.length > 0 ? (
              posts.map((post, i) => (
                <motion.div
                  key={post.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
                >
                  <PostCard post={post as any} onVote={(value) => voteMutation.mutate({ id: post.id, value })} />
                </motion.div>
              ))
            ) : (
              <EmptyState
                icon={<FileText size={28} />}
                title="No posts yet"
                description="This user hasn't posted anything yet."
              />
            )}
          </>
        )}

        {activeTab === 'comments' && (
          <>
            {commentsLoading ? (
              Array.from({ length: 2 }).map((_, i) => (
                <Card key={i} className="p-4 space-y-2">
                  <Skeleton className="w-48 h-3" />
                  <Skeleton className="w-full h-12" />
                </Card>
              ))
            ) : comments.length > 0 ? (
              comments.map((comment, i) => (
                <motion.div
                  key={comment.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.3, delay: Math.min(i * 0.05, 0.3) }}
                >
                  <Card className="p-4">
                    <p className="text-sm text-text-secondary">{comment.content}</p>
                    <p className="text-xs text-text-tertiary mt-2">
                      {formatNumber(comment.score)} points &bull; {formatTimeAgo(comment.created_at)}
                    </p>
                  </Card>
                </motion.div>
              ))
            ) : (
              <EmptyState
                icon={<MessageSquare size={28} />}
                title="No comments yet"
                description="This user hasn't commented on anything yet."
              />
            )}
          </>
        )}
      </div>
    </div>
  );
}
