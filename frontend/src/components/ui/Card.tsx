'use client';

import { cn } from '@/lib/utils';

interface CardProps {
  children: React.ReactNode;
  className?: string;
  hover?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg';
  onClick?: () => void;
}

const paddings = {
  none: '',
  sm: 'p-3',
  md: 'p-4',
  lg: 'p-6',
};

export function Card({
  children,
  className,
  hover = false,
  padding = 'md',
  onClick,
}: CardProps) {
  return (
    <div
      className={cn(
        'bg-surface rounded-2xl border border-surface-border',
        'transition-all duration-200',
        paddings[padding],
        hover && 'hover:bg-surface-hover hover:border-surface-hover cursor-pointer',
        onClick && 'cursor-pointer',
        className
      )}
      onClick={onClick}
    >
      {children}
    </div>
  );
}
