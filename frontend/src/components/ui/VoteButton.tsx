'use client';

import { useState } from 'react';
import { ArrowBigUp, ArrowBigDown } from 'lucide-react';
import { cn, formatNumber } from '@/lib/utils';

interface VoteButtonProps {
  score: number;
  userVote?: number | null;
  onVote: (value: number) => void;
  orientation?: 'vertical' | 'horizontal';
  size?: 'sm' | 'md';
}

export function VoteButton({
  score,
  userVote,
  onVote,
  orientation = 'horizontal',
  size = 'md',
}: VoteButtonProps) {
  const [optimisticVote, setOptimisticVote] = useState<number | null>(null);
  const currentVote = optimisticVote ?? userVote ?? 0;
  const displayScore = score + (optimisticVote !== null ? optimisticVote - (userVote ?? 0) : 0);

  const handleVote = (value: number) => {
    const newVote = currentVote === value ? 0 : value;
    setOptimisticVote(newVote);
    onVote(newVote);
  };

  const iconSize = size === 'sm' ? 16 : 20;

  return (
    <div
      className={cn(
        'flex items-center gap-0.5 rounded-xl bg-bg-tertiary border border-surface-border',
        orientation === 'vertical' ? 'flex-col py-1 px-1.5' : 'flex-row py-0.5 px-1'
      )}
    >
      <button
        onClick={() => handleVote(1)}
        className={cn(
          'p-1 rounded-lg transition-all duration-150',
          currentVote === 1
            ? 'text-accent bg-accent-muted'
            : 'text-text-tertiary hover:text-accent hover:bg-accent-muted/50'
        )}
        aria-label="Upvote"
      >
        <ArrowBigUp
          size={iconSize}
          fill={currentVote === 1 ? 'currentColor' : 'none'}
        />
      </button>
      <span
        className={cn(
          'text-xs font-semibold tabular-nums min-w-[1.5rem] text-center',
          currentVote === 1 && 'text-accent',
          currentVote === -1 && 'text-danger',
          currentVote === 0 && 'text-text-secondary'
        )}
      >
        {formatNumber(displayScore)}
      </span>
      <button
        onClick={() => handleVote(-1)}
        className={cn(
          'p-1 rounded-lg transition-all duration-150',
          currentVote === -1
            ? 'text-danger bg-danger-muted'
            : 'text-text-tertiary hover:text-danger hover:bg-danger-muted/50'
        )}
        aria-label="Downvote"
      >
        <ArrowBigDown
          size={iconSize}
          fill={currentVote === -1 ? 'currentColor' : 'none'}
        />
      </button>
    </div>
  );
}
