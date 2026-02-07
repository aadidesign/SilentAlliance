'use client';

import { cn } from '@/lib/utils';

interface Tab {
  id: string;
  label: string;
  icon?: React.ReactNode;
  count?: number;
}

interface TabsProps {
  tabs: Tab[];
  activeTab: string;
  onChange: (id: string) => void;
  className?: string;
}

export function Tabs({ tabs, activeTab, onChange, className }: TabsProps) {
  return (
    <div
      className={cn(
        'flex items-center gap-1 p-1 bg-bg-tertiary rounded-xl border border-surface-border',
        className
      )}
    >
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => onChange(tab.id)}
          className={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium',
            'transition-all duration-200',
            activeTab === tab.id
              ? 'bg-surface text-text-primary shadow-sm'
              : 'text-text-tertiary hover:text-text-secondary'
          )}
        >
          {tab.icon}
          {tab.label}
          {tab.count !== undefined && (
            <span
              className={cn(
                'text-xs px-1.5 py-0.5 rounded-md',
                activeTab === tab.id
                  ? 'bg-accent-muted text-accent'
                  : 'bg-surface text-text-tertiary'
              )}
            >
              {tab.count}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}
