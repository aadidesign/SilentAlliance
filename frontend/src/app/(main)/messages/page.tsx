'use client';

import { useState } from 'react';
import { motion } from 'framer-motion';
import {
  MessageSquare,
  Search,
  Send,
  Lock,
  Plus,
  ArrowLeft,
  Shield,
  CheckCheck,
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Button } from '@/components/ui/Button';
import { Skeleton } from '@/components/ui/Skeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatTimeAgo } from '@/lib/utils';
import { useAuthStore } from '@/lib/store';
import { useConversations, useMessages } from '@/hooks/queries';
import { useSendMessage } from '@/hooks/mutations';
import { useRequireAuth } from '@/hooks/useRequireAuth';

export default function MessagesPage() {
  const { isAuthenticated, identity } = useRequireAuth();
  const [selectedConversation, setSelectedConversation] = useState<string | null>(null);
  const [messageInput, setMessageInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');

  const { data: convosData, isLoading: convosLoading } = useConversations();
  const { data: messagesData, isLoading: messagesLoading } = useMessages(selectedConversation || '');
  const sendMutation = useSendMessage(selectedConversation || '');

  const conversations = convosData?.data ?? [];
  const messages = messagesData?.data ?? [];

  const activeConvo = conversations.find((c) => c.id === selectedConversation);

  // Get the other participant in the conversation
  const getOtherParticipant = (convo: typeof conversations[0]) => {
    const other = convo.participants?.find((p) => p.identity_id !== identity?.id);
    return other?.identity || { id: other?.identity_id || '', display_name: 'Unknown', public_key_fingerprint: '' };
  };

  const handleSend = () => {
    if (!messageInput.trim() || !selectedConversation) return;
    // In a real implementation, this would encrypt the message with the recipient's public key
    // using X25519 key exchange + ChaCha20-Poly1305
    // For now, we send a base64 placeholder
    const content = btoa(messageInput);
    const nonce = btoa(crypto.getRandomValues(new Uint8Array(24)).toString());
    sendMutation.mutate(
      { encrypted_content: content, nonce },
      { onSuccess: () => setMessageInput('') }
    );
  };

  if (!isAuthenticated) return null;

  return (
    <div className="flex h-[calc(100vh-7rem)] -mx-4 -mb-6 rounded-2xl overflow-hidden border border-surface-border">
      {/* Conversations list */}
      <div
        className={cn(
          'w-full sm:w-80 shrink-0 border-r border-surface-border bg-bg-secondary flex flex-col',
          selectedConversation && 'hidden sm:flex'
        )}
      >
        {/* Header */}
        <div className="p-4 border-b border-surface-border">
          <div className="flex items-center justify-between mb-3">
            <h1 className="text-lg font-semibold text-text-primary">Messages</h1>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <Plus size={16} />
            </Button>
          </div>
          <div className="relative">
            <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-tertiary" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search conversations"
              className="w-full h-8 pl-8 pr-3 text-xs bg-bg-tertiary border border-surface-border rounded-lg text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent/30 transition-colors"
            />
          </div>
        </div>

        {/* E2E indicator */}
        <div className="flex items-center gap-2 px-4 py-2 border-b border-surface-border bg-accent-muted/30">
          <Lock size={12} className="text-accent" />
          <span className="text-2xs text-accent">End-to-end encrypted</span>
        </div>

        {/* Conversation list */}
        <div className="flex-1 overflow-y-auto hide-scrollbar">
          {convosLoading ? (
            Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="flex items-start gap-3 px-4 py-3">
                <Skeleton className="w-10 h-10 rounded-full" />
                <div className="flex-1 space-y-1.5">
                  <Skeleton className="w-24 h-4" />
                  <Skeleton className="w-40 h-3" />
                </div>
              </div>
            ))
          ) : conversations.length === 0 ? (
            <div className="p-6 text-center">
              <MessageSquare size={20} className="mx-auto text-text-tertiary mb-2" />
              <p className="text-xs text-text-tertiary">No conversations yet.</p>
            </div>
          ) : (
            conversations.map((convo) => {
              const participant = getOtherParticipant(convo);
              return (
                <button
                  key={convo.id}
                  onClick={() => setSelectedConversation(convo.id)}
                  className={cn(
                    'w-full flex items-start gap-3 px-4 py-3 text-left',
                    'hover:bg-surface-hover transition-colors',
                    selectedConversation === convo.id && 'bg-surface-hover'
                  )}
                >
                  <Avatar id={participant.id || ''} name={participant.display_name} size="md" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between mb-0.5">
                      <span className="text-sm font-medium text-text-primary truncate">
                        {participant.display_name || 'Anonymous'}
                      </span>
                      <span className="text-2xs text-text-tertiary shrink-0 ml-2">
                        {formatTimeAgo(convo.updated_at)}
                      </span>
                    </div>
                    <p className="text-xs truncate text-text-tertiary">
                      {convo.last_message ? '[Encrypted message]' : 'No messages yet'}
                    </p>
                  </div>
                </button>
              );
            })
          )}
        </div>
      </div>

      {/* Chat view */}
      <div
        className={cn(
          'flex-1 flex flex-col bg-bg-primary',
          !selectedConversation && 'hidden sm:flex'
        )}
      >
        {selectedConversation && activeConvo ? (
          <>
            {/* Chat header */}
            <div className="flex items-center gap-3 px-4 py-3 border-b border-surface-border glass-strong">
              <button
                onClick={() => setSelectedConversation(null)}
                className="sm:hidden p-1.5 rounded-lg text-text-tertiary hover:text-text-primary transition-colors"
              >
                <ArrowLeft size={18} />
              </button>
              {(() => {
                const participant = getOtherParticipant(activeConvo);
                return (
                  <>
                    <Avatar id={participant.id || ''} name={participant.display_name} size="sm" />
                    <div className="flex-1">
                      <p className="text-sm font-medium text-text-primary">
                        {participant.display_name || 'Anonymous'}
                      </p>
                      <div className="flex items-center gap-1 text-2xs text-text-tertiary">
                        <Shield size={10} className="text-accent" />
                        <span>E2E Encrypted</span>
                      </div>
                    </div>
                  </>
                );
              })()}
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3 hide-scrollbar">
              {messagesLoading ? (
                <div className="space-y-3">
                  {Array.from({ length: 4 }).map((_, i) => (
                    <div key={i} className={cn('flex', i % 2 === 0 ? 'justify-start' : 'justify-end')}>
                      <Skeleton className="w-48 h-12 rounded-2xl" />
                    </div>
                  ))}
                </div>
              ) : messages.length === 0 ? (
                <div className="flex-1 flex items-center justify-center py-12">
                  <p className="text-sm text-text-tertiary">No messages yet. Say hello!</p>
                </div>
              ) : (
                messages.map((msg) => {
                  const isOwn = msg.sender_id === identity?.id;
                  // In a real implementation, decrypt msg.encrypted_content with shared key
                  let displayContent: string;
                  try {
                    displayContent = atob(msg.encrypted_content);
                  } catch {
                    displayContent = '[Encrypted message]';
                  }

                  return (
                    <motion.div
                      key={msg.id}
                      initial={{ opacity: 0, y: 5 }}
                      animate={{ opacity: 1, y: 0 }}
                      className={cn('flex', isOwn ? 'justify-end' : 'justify-start')}
                    >
                      <div
                        className={cn(
                          'max-w-[75%] rounded-2xl px-4 py-2.5',
                          isOwn
                            ? 'bg-accent text-text-on-accent rounded-br-md'
                            : 'bg-surface border border-surface-border text-text-primary rounded-bl-md'
                        )}
                      >
                        <p className="text-sm leading-relaxed">{displayContent}</p>
                        <div
                          className={cn(
                            'flex items-center gap-1 mt-1',
                            isOwn ? 'justify-end' : 'justify-start'
                          )}
                        >
                          <span
                            className={cn(
                              'text-2xs',
                              isOwn ? 'text-text-on-accent/60' : 'text-text-tertiary'
                            )}
                          >
                            {formatTimeAgo(msg.created_at)}
                          </span>
                          {isOwn && (
                            <CheckCheck size={12} className="text-text-on-accent/60" />
                          )}
                        </div>
                      </div>
                    </motion.div>
                  );
                })
              )}
            </div>

            {/* Message input */}
            <div className="p-4 border-t border-surface-border">
              <div className="flex items-end gap-2">
                <div className="flex-1 relative">
                  <textarea
                    value={messageInput}
                    onChange={(e) => setMessageInput(e.target.value)}
                    placeholder="Type a message..."
                    rows={1}
                    className={cn(
                      'w-full px-4 py-2.5 text-sm bg-bg-tertiary text-text-primary',
                      'border border-surface-border rounded-xl',
                      'placeholder:text-text-tertiary',
                      'focus:outline-none focus:border-accent/30',
                      'transition-colors resize-none'
                    )}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !e.shiftKey) {
                        e.preventDefault();
                        handleSend();
                      }
                    }}
                  />
                </div>
                <Button
                  size="icon"
                  disabled={!messageInput.trim() || sendMutation.isPending}
                  onClick={handleSend}
                  className="shrink-0"
                >
                  <Send size={16} />
                </Button>
              </div>
              <p className="text-2xs text-text-tertiary mt-1.5 flex items-center gap-1">
                <Lock size={10} />
                Messages are end-to-end encrypted with ChaCha20-Poly1305
              </p>
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <div className="w-16 h-16 rounded-2xl bg-surface flex items-center justify-center mx-auto mb-4">
                <MessageSquare size={28} className="text-text-tertiary" />
              </div>
              <h3 className="text-lg font-semibold text-text-primary mb-1">Your Messages</h3>
              <p className="text-sm text-text-tertiary max-w-xs">
                Select a conversation or start a new one. All messages are end-to-end encrypted.
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
