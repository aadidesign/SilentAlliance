'use client';

import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  hint?: string;
  leftIcon?: React.ReactNode;
  rightElement?: React.ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, hint, leftIcon, rightElement, className, id, ...props }, ref) => {
    const inputId = id || label?.toLowerCase().replace(/\s+/g, '-');

    return (
      <div className="space-y-1.5">
        {label && (
          <label
            htmlFor={inputId}
            className="block text-sm font-medium text-text-secondary"
          >
            {label}
          </label>
        )}
        <div className="relative">
          {leftIcon && (
            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-text-tertiary">
              {leftIcon}
            </div>
          )}
          <input
            ref={ref}
            id={inputId}
            className={cn(
              'w-full h-10 px-3 bg-bg-tertiary text-text-primary text-sm',
              'border border-surface-border rounded-xl',
              'placeholder:text-text-tertiary',
              'focus:outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/20',
              'transition-all duration-200',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              leftIcon && 'pl-10',
              rightElement && 'pr-10',
              error && 'border-danger/50 focus:border-danger/50 focus:ring-danger/20',
              className
            )}
            {...props}
          />
          {rightElement && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2">
              {rightElement}
            </div>
          )}
        </div>
        {error && (
          <p className="text-xs text-danger mt-1">{error}</p>
        )}
        {hint && !error && (
          <p className="text-xs text-text-tertiary mt-1">{hint}</p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

// ==================== Textarea ====================

interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  hint?: string;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ label, error, hint, className, id, ...props }, ref) => {
    const inputId = id || label?.toLowerCase().replace(/\s+/g, '-');

    return (
      <div className="space-y-1.5">
        {label && (
          <label
            htmlFor={inputId}
            className="block text-sm font-medium text-text-secondary"
          >
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={inputId}
          className={cn(
            'w-full px-3 py-2.5 bg-bg-tertiary text-text-primary text-sm',
            'border border-surface-border rounded-xl',
            'placeholder:text-text-tertiary',
            'focus:outline-none focus:border-accent/50 focus:ring-1 focus:ring-accent/20',
            'transition-all duration-200 resize-y min-h-[100px]',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            error && 'border-danger/50 focus:border-danger/50 focus:ring-danger/20',
            className
          )}
          {...props}
        />
        {error && <p className="text-xs text-danger mt-1">{error}</p>}
        {hint && !error && <p className="text-xs text-text-tertiary mt-1">{hint}</p>}
      </div>
    );
  }
);

Textarea.displayName = 'Textarea';
