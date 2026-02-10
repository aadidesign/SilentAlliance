'use client';

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
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatTimeAgo } from '@/lib/utils';
import { useNotifications } from '@/hooks/queries';
import { useMarkNotificationRead, useMarkAllNotificationsRead } from '@/hooks/mutations';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import type { NotificationType } from '@/types';

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

function getNotifTitle(type: NotificationType, payload: Record<string, unknown>): string {
  const actor = (payload.actor_name as string) || 'Someone';
  switch (type) {
    case 'post_reply': return `${actor} commented on your post`;
    case 'comment_reply': return `${actor} replied to your comment`;
    case 'mention': return `${actor} mentioned you`;
    case 'new_message': return `New message from ${actor}`;
    case 'space_invite': return `You were invited to ${(payload.space_name as string) || 'a space'}`;
    case 'moderator_action': return 'Moderator action on your content';
    case 'system_alert': return (payload.title as string) || 'System notification';
  }
}

function getNotifLink(type: NotificationType, payload: Record<string, unknown>): string {
  switch (type) {
    case 'post_reply':
    case 'comment_reply':
    case 'mention':
      return `/post/${(payload.post_id as string) || ''}`;
    case 'new_message':
      return '/messages';
    case 'space_invite':
      return `/s/${(payload.space_slug as string) || ''}`;
    default:
      return '#';
  }
}

export default function NotificationsPage() {
  const { isAuthenticated } = useRequireAuth();
  const { data, isLoading } = useNotifications();
  const markReadMutation = useMarkNotificationRead();
  const markAllReadMutation = useMarkAllNotificationsRead();

  const notifications = data?.data ?? [];
  const unreadCount = notifications.filter((n) => !n.is_read).length;

  if (!isAuthenticated) return null;

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
          <Button
            variant="ghost"
            size="sm"
            onClick={() => markAllReadMutation.mutate()}
            isLoading={markAllReadMutation.isPending}
            leftIcon={<Check size={14} />}
          >
            Mark all read
          </Button>
        )}
      </div>

      {/* Notifications */}
      <Card padding="none">
        {isLoading ? (
          <div className="divide-y divide-surface-border/50">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-start gap-3 px-4 py-3.5">
                <Skeleton className="w-9 h-9 rounded-xl" />
                <div className="flex-1 space-y-1.5">
                  <Skeleton className="w-3/4 h-4" />
                  <Skeleton className="w-1/2 h-3" />
                </div>
              </div>
            ))}
          </div>
        ) : notifications.length > 0 ? (
          <div className="divide-y divide-surface-border/50">
            {notifications.map((notif, i) => (
              <motion.div
                key={notif.id}
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.2, delay: Math.min(i * 0.03, 0.3) }}
              >
                <Link
                  href={getNotifLink(notif.notification_type, notif.payload)}
                  onClick={() => {
                    if (!notif.is_read) {
                      markReadMutation.mutate(notif.id);
                    }
                  }}
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
                      notifColors[notif.notification_type]
                    )}
                  >
                    {notifIcons[notif.notification_type]}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <p
                      className={cn(
                        'text-sm leading-snug',
                        !notif.is_read ? 'text-text-primary font-medium' : 'text-text-secondary'
                      )}
                    >
                      {getNotifTitle(notif.notification_type, notif.payload)}
                    </p>
                    {notif.payload.body && (
                      <p className="text-xs text-text-tertiary mt-0.5 truncate">
                        {notif.payload.body as string}
                      </p>
                    )}
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
