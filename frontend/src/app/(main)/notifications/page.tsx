'use client';

import { useState } from 'react';
import Link from 'next/link';
import { motion } from 'framer-motion';
import {
  Bell,
  MessageSquare,
  Reply,
  ArrowBigUp,
  AtSign,
  Shield,
  Users,
  AlertTriangle,
  Check,
  Trash2,
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatTimeAgo } from '@/lib/utils';
import type { NotificationType } from '@/types';

interface SampleNotification {
  id: string;
  type: NotificationType;
  title: string;
  body: string;
  link: string;
  is_read: boolean;
  created_at: string;
  actor?: { id: string; name: string };
}

const notifIcons: Record<NotificationType, React.ReactNode> = {
  post_reply: <MessageSquare size={16} />,
  comment_reply: <Reply size={16} />,
  mention: <AtSign size={16} />,
  new_message: <MessageSquare size={16} />,
  space_invite: <Users size={16} />,
  moderator_action: <Shield size={16} />,
  system_alert: <AlertTriangle size={16} />,
};

const notifColors: Record<NotificationType, string> = {
  post_reply: 'text-accent bg-accent-muted',
  comment_reply: 'text-accent bg-accent-muted',
  mention: 'text-accent-hover bg-accent-muted',
  new_message: 'text-success bg-success-muted',
  space_invite: 'text-warning bg-warning-muted',
  moderator_action: 'text-danger bg-danger-muted',
  system_alert: 'text-warning bg-warning-muted',
};

const sampleNotifications: SampleNotification[] = [
  {
    id: 'n1',
    type: 'comment_reply',
    title: 'CryptoEnthusiast replied to your comment',
    body: '"Great analysis. I think the most impactful thing we can do..."',
    link: '/post/p1',
    is_read: false,
    created_at: new Date(Date.now() - 600000).toISOString(),
    actor: { id: 'u1', name: 'CryptoEnthusiast' },
  },
  {
    id: 'n2',
    type: 'mention',
    title: 'FOSSAdvocate mentioned you',
    body: '"@PrivacyAdvocate wrote a great post about this topic..."',
    link: '/post/p2',
    is_read: false,
    created_at: new Date(Date.now() - 3600000).toISOString(),
    actor: { id: 'u2', name: 'FOSSAdvocate' },
  },
  {
    id: 'n3',
    type: 'new_message',
    title: 'New message from CryptoEngineer',
    body: '"Can you share the repo?"',
    link: '/messages',
    is_read: false,
    created_at: new Date(Date.now() - 3600000 * 2).toISOString(),
    actor: { id: 'u3', name: 'CryptoEngineer' },
  },
  {
    id: 'n4',
    type: 'post_reply',
    title: 'New comment on your post',
    body: '"I\'d add that we should also be supporting open-source alternatives..."',
    link: '/post/p1',
    is_read: true,
    created_at: new Date(Date.now() - 86400000).toISOString(),
    actor: { id: 'u4', name: 'Anonymous' },
  },
  {
    id: 'n5',
    type: 'space_invite',
    title: 'You were invited to s/whistleblowers',
    body: 'A private space for verified whistleblowers.',
    link: '/s/whistleblowers',
    is_read: true,
    created_at: new Date(Date.now() - 86400000 * 2).toISOString(),
  },
  {
    id: 'n6',
    type: 'system_alert',
    title: 'Security update',
    body: 'We\'ve enhanced our encryption protocols. No action required.',
    link: '#',
    is_read: true,
    created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
  },
];

export default function NotificationsPage() {
  const [notifications, setNotifications] = useState(sampleNotifications);
  const unreadCount = notifications.filter((n) => !n.is_read).length;

  const markAllRead = () => {
    setNotifications((prev) => prev.map((n) => ({ ...n, is_read: true })));
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h1 className="text-2xl font-bold text-text-primary">Notifications</h1>
          {unreadCount > 0 && (
            <span className="text-xs font-medium text-accent bg-accent-muted px-2 py-0.5 rounded-md">
              {unreadCount} new
            </span>
          )}
        </div>
        {unreadCount > 0 && (
          <Button variant="ghost" size="sm" onClick={markAllRead} leftIcon={<Check size={14} />}>
            Mark all read
          </Button>
        )}
      </div>

      {/* Notifications */}
      <Card padding="none">
        {notifications.length > 0 ? (
          <div className="divide-y divide-surface-border/50">
            {notifications.map((notif, i) => (
              <motion.div
                key={notif.id}
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2, delay: Math.min(i * 0.03, 0.3) }}
              >
                <Link
                  href={notif.link}
                  className={cn(
                    'flex items-start gap-3 px-4 py-3.5',
                    'hover:bg-surface-hover transition-colors',
                    !notif.is_read && 'bg-accent-muted/5'
                  )}
                >
                  {/* Icon */}
                  <div
                    className={cn(
                      'w-9 h-9 rounded-xl flex items-center justify-center shrink-0 mt-0.5',
                      notifColors[notif.type]
                    )}
                  >
                    {notifIcons[notif.type]}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <p
                      className={cn(
                        'text-sm leading-snug',
                        !notif.is_read ? 'text-text-primary font-medium' : 'text-text-secondary'
                      )}
                    >
                      {notif.title}
                    </p>
                    <p className="text-xs text-text-tertiary mt-0.5 truncate">
                      {notif.body}
                    </p>
                    <p className="text-2xs text-text-tertiary mt-1">
                      {formatTimeAgo(notif.created_at)}
                    </p>
                  </div>

                  {/* Unread dot */}
                  {!notif.is_read && (
                    <div className="w-2 h-2 rounded-full bg-accent shrink-0 mt-2" />
                  )}
                </Link>
              </motion.div>
            ))}
          </div>
        ) : (
          <EmptyState
            icon={<Bell size={28} />}
            title="No notifications"
            description="You're all caught up. We'll notify you when something happens."
          />
        )}
      </Card>
    </div>
  );
}
