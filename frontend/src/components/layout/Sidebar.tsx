'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import {
  Home,
  Flame,
  TrendingUp,
  Clock,
  Users,
  Plus,
  MessageSquare,
  Bell,
  Settings,
  Shield,
  Hash,
  Globe,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/lib/utils';
import { useAuthStore, useUIStore } from '@/lib/store';
import { Button } from '@/components/ui/Button';

interface NavItem {
  label: string;
  href: string;
  icon: React.ReactNode;
  auth?: boolean;
}

const mainNav: NavItem[] = [
  { label: 'Home Feed', href: '/feed', icon: <Home size={18} /> },
  { label: 'Popular', href: '/popular', icon: <Flame size={18} /> },
  { label: 'All', href: '/all', icon: <Globe size={18} /> },
  { label: 'Trending', href: '/trending', icon: <TrendingUp size={18} /> },
];

const personalNav: NavItem[] = [
  { label: 'Messages', href: '/messages', icon: <MessageSquare size={18} />, auth: true },
  { label: 'Notifications', href: '/notifications', icon: <Bell size={18} />, auth: true },
  { label: 'Settings', href: '/settings', icon: <Settings size={18} />, auth: true },
];

// Placeholder spaces - in production these would come from the API
const sampleSpaces = [
  { name: 'crypto', slug: 'crypto', members: '12.4K' },
  { name: 'privacy', slug: 'privacy', members: '8.2K' },
  { name: 'technology', slug: 'technology', members: '34.1K' },
  { name: 'whistleblowers', slug: 'whistleblowers', members: '5.7K' },
  { name: 'defi', slug: 'defi', members: '9.8K' },
];

export function Sidebar() {
  const pathname = usePathname();
  const { isAuthenticated } = useAuthStore();
  const { mobileMenuOpen, closeMobileMenu } = useUIStore();

  const sidebarContent = (
    <div className="flex flex-col h-full">
      {/* Main navigation */}
      <nav className="px-3 py-4 space-y-0.5">
        <p className="px-3 mb-2 text-xs font-semibold text-text-tertiary uppercase tracking-wider">
          Feeds
        </p>
        {mainNav.map((item) => (
          <Link
            key={item.href}
            href={item.href}
            onClick={closeMobileMenu}
            className={cn(
              'flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium',
              'transition-all duration-150',
              pathname === item.href
                ? 'bg-accent-muted text-accent'
                : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
            )}
          >
            {item.icon}
            {item.label}
          </Link>
        ))}
      </nav>

      {/* Divider */}
      <div className="mx-4 border-t border-surface-border" />

      {/* Spaces */}
      <nav className="px-3 py-4 space-y-0.5 flex-1 overflow-y-auto hide-scrollbar">
        <div className="flex items-center justify-between px-3 mb-2">
          <p className="text-xs font-semibold text-text-tertiary uppercase tracking-wider">
            Spaces
          </p>
          {isAuthenticated && (
            <Link href="/create-space">
              <button className="p-1 rounded-md text-text-tertiary hover:text-accent transition-colors">
                <Plus size={14} />
              </button>
            </Link>
          )}
        </div>
        {sampleSpaces.map((space) => (
          <Link
            key={space.slug}
            href={`/s/${space.slug}`}
            onClick={closeMobileMenu}
            className={cn(
              'flex items-center gap-3 px-3 py-2 rounded-xl text-sm',
              'transition-all duration-150',
              pathname === `/s/${space.slug}`
                ? 'bg-accent-muted text-accent'
                : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
            )}
          >
            <div className="w-6 h-6 rounded-md bg-surface flex items-center justify-center shrink-0">
              <Hash size={12} className="text-text-tertiary" />
            </div>
            <span className="truncate">{space.name}</span>
            <span className="text-2xs text-text-tertiary ml-auto shrink-0">
              {space.members}
            </span>
          </Link>
        ))}
      </nav>

      {/* Divider */}
      <div className="mx-4 border-t border-surface-border" />

      {/* Personal nav */}
      {isAuthenticated && (
        <nav className="px-3 py-4 space-y-0.5">
          <p className="px-3 mb-2 text-xs font-semibold text-text-tertiary uppercase tracking-wider">
            Personal
          </p>
          {personalNav
            .filter((item) => !item.auth || isAuthenticated)
            .map((item) => (
              <Link
                key={item.href}
                href={item.href}
                onClick={closeMobileMenu}
                className={cn(
                  'flex items-center gap-3 px-3 py-2 rounded-xl text-sm font-medium',
                  'transition-all duration-150',
                  pathname === item.href
                    ? 'bg-accent-muted text-accent'
                    : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
                )}
              >
                {item.icon}
                {item.label}
              </Link>
            ))}
        </nav>
      )}

    </div>
  );

  return (
    <>
      {/* Desktop sidebar */}
      <aside className="hidden lg:block fixed left-0 top-16 bottom-0 w-64 border-r border-surface-border bg-bg-secondary overflow-y-auto hide-scrollbar">
        {sidebarContent}
      </aside>

      {/* Mobile sidebar overlay */}
      <AnimatePresence>
        {mobileMenuOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="fixed inset-0 z-40 bg-bg-overlay backdrop-blur-sm lg:hidden"
              onClick={closeMobileMenu}
            />
            <motion.aside
              initial={{ x: -280 }}
              animate={{ x: 0 }}
              exit={{ x: -280 }}
              transition={{ type: 'spring', damping: 25, stiffness: 200 }}
              className="fixed left-0 top-0 bottom-0 w-72 z-50 bg-bg-secondary border-r border-surface-border lg:hidden overflow-y-auto"
            >
              {/* Mobile header */}
              <div className="h-16 flex items-center gap-2.5 px-5 border-b border-surface-border">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-accent-secondary flex items-center justify-center">
                  <Shield size={18} className="text-text-on-accent" />
                </div>
                <span className="font-bold text-lg text-text-primary">
                  Silent<span className="gradient-text">Alliance</span>
                </span>
              </div>
              {sidebarContent}
            </motion.aside>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
