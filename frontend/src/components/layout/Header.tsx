'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import {
  Search,
  Bell,
  MessageSquare,
  Plus,
  Menu,
  Shield,
  LogOut,
  Settings,
  User,
  ChevronDown,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/utils';
import { useAuthStore, useUIStore } from '@/lib/store';
import { Avatar } from '@/components/ui/Avatar';
import { Button } from '@/components/ui/Button';

export function Header() {
  const router = useRouter();
  const { identity, isAuthenticated, logout } = useAuthStore();
  const { toggleMobileMenu } = useUIStore();
  const [searchFocused, setSearchFocused] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    if (searchQuery.trim()) {
      router.push(`/search?q=${encodeURIComponent(searchQuery.trim())}`);
    }
  };

  return (
    <header className="fixed top-0 left-0 right-0 z-40 h-16">
      <div className="h-full glass-strong">
        <div className="h-full max-w-[1800px] mx-auto px-4 flex items-center gap-3">
          {/* Mobile menu button */}
          <button
            onClick={toggleMobileMenu}
            className="lg:hidden p-2 rounded-xl text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
          >
            <Menu size={20} />
          </button>

          {/* Logo */}
          <Link href={isAuthenticated ? '/feed' : '/'} className="flex items-center gap-2.5 shrink-0">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-accent-secondary flex items-center justify-center">
              <Shield size={18} className="text-text-on-accent" />
            </div>
            <span className="font-bold text-lg text-text-primary hidden sm:block">
              Silent<span className="gradient-text">Alliance</span>
            </span>
          </Link>

          {/* Search */}
          <form onSubmit={handleSearch} className="flex-1 max-w-xl mx-4">
            <div className="relative">
              <Search
                size={16}
                className={cn(
                  'absolute left-3 top-1/2 -translate-y-1/2 transition-colors duration-200',
                  searchFocused ? 'text-accent' : 'text-text-tertiary'
                )}
              />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search spaces, posts..."
                onFocus={() => setSearchFocused(true)}
                onBlur={() => setSearchFocused(false)}
                className={cn(
                  'w-full h-9 pl-9 pr-4 text-sm rounded-xl',
                  'bg-bg-tertiary border border-surface-border',
                  'text-text-primary placeholder:text-text-tertiary',
                  'focus:outline-none focus:border-accent/30 focus:bg-bg-secondary',
                  'transition-all duration-200'
                )}
              />
            </div>
          </form>

          {/* Right side actions */}
          <div className="flex items-center gap-1.5">
            {isAuthenticated ? (
              <>
                {/* Create post */}
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => router.push('/submit')}
                  className="hidden sm:flex"
                >
                  <Plus size={18} />
                </Button>

                {/* Messages */}
                <Link href="/messages">
                  <Button variant="ghost" size="icon">
                    <MessageSquare size={18} />
                  </Button>
                </Link>

                {/* Notifications */}
                <Link href="/notifications">
                  <Button variant="ghost" size="icon" className="relative">
                    <Bell size={18} />
                    {/* Unread indicator */}
                    <span className="absolute top-1.5 right-1.5 w-2 h-2 bg-accent rounded-full" />
                  </Button>
                </Link>

                {/* Profile dropdown */}
                <div className="relative ml-1">
                  <button
                    onClick={() => setProfileOpen(!profileOpen)}
                    className={cn(
                      'flex items-center gap-2 px-2 py-1.5 rounded-xl',
                      'hover:bg-surface-hover transition-colors',
                      profileOpen && 'bg-surface-hover'
                    )}
                  >
                    <Avatar
                      id={identity?.id || ''}
                      name={identity?.display_name}
                      size="sm"
                    />
                    <ChevronDown
                      size={14}
                      className={cn(
                        'text-text-tertiary transition-transform duration-200 hidden sm:block',
                        profileOpen && 'rotate-180'
                      )}
                    />
                  </button>

                  <AnimatePresence>
                    {profileOpen && (
                      <>
                        <div
                          className="fixed inset-0 z-40"
                          onClick={() => setProfileOpen(false)}
                        />
                        <motion.div
                          initial={{ opacity: 0, y: 5, scale: 0.97 }}
                          animate={{ opacity: 1, y: 0, scale: 1 }}
                          exit={{ opacity: 0, y: 5, scale: 0.97 }}
                          transition={{ duration: 0.15 }}
                          className="absolute right-0 top-full mt-2 w-56 z-50 bg-bg-elevated rounded-xl border border-surface-border shadow-surface-lg overflow-hidden"
                        >
                          {/* User info */}
                          <div className="px-4 py-3 border-b border-surface-border">
                            <p className="text-sm font-medium text-text-primary truncate">
                              {identity?.display_name || 'Anonymous'}
                            </p>
                            <p className="text-xs text-text-tertiary font-mono mt-0.5 truncate">
                              {identity?.public_key_fingerprint?.slice(0, 16)}...
                            </p>
                          </div>

                          {/* Menu items */}
                          <div className="py-1">
                            <Link
                              href={`/u/${identity?.id}`}
                              onClick={() => setProfileOpen(false)}
                              className="flex items-center gap-2.5 px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
                            >
                              <User size={16} />
                              Profile
                            </Link>
                            <Link
                              href="/settings"
                              onClick={() => setProfileOpen(false)}
                              className="flex items-center gap-2.5 px-4 py-2 text-sm text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
                            >
                              <Settings size={16} />
                              Settings
                            </Link>
                          </div>

                          <div className="border-t border-surface-border py-1">
                            <button
                              onClick={() => {
                                setProfileOpen(false);
                                logout();
                                router.push('/');
                              }}
                              className="flex items-center gap-2.5 px-4 py-2 text-sm text-danger hover:bg-danger-muted w-full transition-colors"
                            >
                              <LogOut size={16} />
                              Sign out
                            </button>
                          </div>
                        </motion.div>
                      </>
                    )}
                  </AnimatePresence>
                </div>
              </>
            ) : (
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => router.push('/login')}
                >
                  Log in
                </Button>
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => router.push('/register')}
                >
                  Get Started
                </Button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  );
}
