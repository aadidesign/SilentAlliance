'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  MessageSquare,
  Search,
  Send,
  Lock,
  Plus,
  ArrowLeft,
  Shield,
  Check,
  CheckCheck,
} from 'lucide-react';
import { Avatar } from '@/components/ui/Avatar';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { EmptyState } from '@/components/ui/EmptyState';
import { cn, formatTimeAgo } from '@/lib/utils';
import { useAuthStore } from '@/lib/store';

interface ConversationPreview {
  id: string;
  participant: { id: string; name: string };
  lastMessage: string;
  lastMessageTime: string;
  unread: boolean;
}

interface ChatMessage {
  id: string;
  senderId: string;
  content: string;
  timestamp: string;
  isOwn: boolean;
}

const sampleConversations: ConversationPreview[] = [
  {
    id: '1',
    participant: { id: 'u1', name: 'CryptoEngineer' },
    lastMessage: 'That ZK-proof approach is really interesting. Can you share the repo?',
    lastMessageTime: new Date(Date.now() - 600000).toISOString(),
    unread: true,
  },
  {
    id: '2',
    participant: { id: 'u2', name: 'PrivacyAdvocate' },
    lastMessage: 'Thanks for the Signal recommendation. Switched the whole team over.',
    lastMessageTime: new Date(Date.now() - 3600000 * 3).toISOString(),
    unread: false,
  },
  {
    id: '3',
    participant: { id: 'u3', name: 'Anonymous' },
    lastMessage: 'The documents have been verified. Ready to publish.',
    lastMessageTime: new Date(Date.now() - 86400000).toISOString(),
    unread: false,
  },
];

const sampleMessages: ChatMessage[] = [
  {
    id: 'm1',
    senderId: 'u1',
    content: 'Hey, I saw your post about zero-knowledge proofs.',
    timestamp: new Date(Date.now() - 3600000 * 2).toISOString(),
    isOwn: false,
  },
  {
    id: 'm2',
    senderId: 'me',
    content: 'Yes! I\'ve been working on it for a few months now.',
    timestamp: new Date(Date.now() - 3600000 * 1.8).toISOString(),
    isOwn: true,
  },
  {
    id: 'm3',
    senderId: 'u1',
    content: 'The implementation you described using Groth16 was clever. How are you handling the trusted setup?',
    timestamp: new Date(Date.now() - 3600000 * 1.5).toISOString(),
    isOwn: false,
  },
  {
    id: 'm4',
    senderId: 'me',
    content: 'We\'re using a multi-party ceremony with 128 participants. The circuit is relatively simple so the trusted setup is manageable.',
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    isOwn: true,
  },
  {
    id: 'm5',
    senderId: 'u1',
    content: 'That ZK-proof approach is really interesting. Can you share the repo?',
    timestamp: new Date(Date.now() - 600000).toISOString(),
    isOwn: false,
  },
];

export default function MessagesPage() {
  const { isAuthenticated } = useAuthStore();
  const [selectedConversation, setSelectedConversation] = useState<string | null>(null);
  const [messageInput, setMessageInput] = useState('');
  const [searchQuery, setSearchQuery] = useState('');

  const activeConvo = sampleConversations.find((c) => c.id === selectedConversation);

  if (!isAuthenticated) {
    return (
      <EmptyState
        icon={<Lock size={28} />}
        title="Sign in to view messages"
        description="Your messages are end-to-end encrypted. Sign in to access them."
        action={{ label: 'Sign In', onClick: () => {} }}
      />
    );
  }

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
          {sampleConversations.map((convo) => (
            <button
              key={convo.id}
              onClick={() => setSelectedConversation(convo.id)}
              className={cn(
                'w-full flex items-start gap-3 px-4 py-3 text-left',
                'hover:bg-surface-hover transition-colors',
                selectedConversation === convo.id && 'bg-surface-hover'
              )}
            >
              <Avatar id={convo.participant.id} name={convo.participant.name} size="md" />
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between mb-0.5">
                  <span className="text-sm font-medium text-text-primary truncate">
                    {convo.participant.name}
                  </span>
                  <span className="text-2xs text-text-tertiary shrink-0 ml-2">
                    {formatTimeAgo(convo.lastMessageTime)}
                  </span>
                </div>
                <p
                  className={cn(
                    'text-xs truncate',
                    convo.unread ? 'text-text-secondary font-medium' : 'text-text-tertiary'
                  )}
                >
                  {convo.lastMessage}
                </p>
              </div>
              {convo.unread && (
                <div className="w-2 h-2 rounded-full bg-accent shrink-0 mt-2" />
              )}
            </button>
          ))}
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
              <Avatar
                id={activeConvo.participant.id}
                name={activeConvo.participant.name}
                size="sm"
              />
              <div className="flex-1">
                <p className="text-sm font-medium text-text-primary">
                  {activeConvo.participant.name}
                </p>
                <div className="flex items-center gap-1 text-2xs text-text-tertiary">
                  <Shield size={10} className="text-accent" />
                  <span>E2E Encrypted</span>
                </div>
              </div>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3 hide-scrollbar">
              {sampleMessages.map((msg) => (
                <motion.div
                  key={msg.id}
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: 1, y: 0 }}
                  className={cn(
                    'flex',
                    msg.isOwn ? 'justify-end' : 'justify-start'
                  )}
                >
                  <div
                    className={cn(
                      'max-w-[75%] rounded-2xl px-4 py-2.5',
                      msg.isOwn
                        ? 'bg-accent text-text-on-accent rounded-br-md'
                        : 'bg-surface border border-surface-border text-text-primary rounded-bl-md'
                    )}
                  >
                    <p className="text-sm leading-relaxed">{msg.content}</p>
                    <div
                      className={cn(
                        'flex items-center gap-1 mt-1',
                        msg.isOwn ? 'justify-end' : 'justify-start'
                      )}
                    >
                      <span
                        className={cn(
                          'text-2xs',
                          msg.isOwn ? 'text-text-on-accent/60' : 'text-text-tertiary'
                        )}
                      >
                        {formatTimeAgo(msg.timestamp)}
                      </span>
                      {msg.isOwn && (
                        <CheckCheck size={12} className="text-text-on-accent/60" />
                      )}
                    </div>
                  </div>
                </motion.div>
              ))}
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
                        if (messageInput.trim()) {
                          console.log('Send:', messageInput);
                          setMessageInput('');
                        }
                      }
                    }}
                  />
                </div>
                <Button
                  size="icon"
                  disabled={!messageInput.trim()}
                  onClick={() => {
                    if (messageInput.trim()) {
                      console.log('Send:', messageInput);
                      setMessageInput('');
                    }
                  }}
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
