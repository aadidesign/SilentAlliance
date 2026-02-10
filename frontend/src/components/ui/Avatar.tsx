'use client';

import { cn, generateAvatarGradient } from '@/lib/utils';

interface AvatarProps {
  id: string;
  name?: string | null;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
}

const sizes = {
  xs: 'w-6 h-6 text-2xs',
  sm: 'w-8 h-8 text-xs',
  md: 'w-10 h-10 text-sm',
  lg: 'w-12 h-12 text-base',
  xl: 'w-16 h-16 text-lg',
};

export function Avatar({ id, name, size = 'md', className }: AvatarProps) {
  const gradient = generateAvatarGradient(id);
  const initial = name ? name.charAt(0).toUpperCase() : '?';

  return (
    <div
      className={cn(
        'rounded-full flex items-center justify-center font-semibold text-text-on-accent shrink-0',
        'ring-1 ring-surface-border',
        sizes[size],
        className
      )}
      style={{ background: gradient }}
      title={name || 'Anonymous'}
    >
      {initial}
    </div>
  );
}
