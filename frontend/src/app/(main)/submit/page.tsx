'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  FileText,
  Link2,
  Image,
  ArrowRight,
  ChevronDown,
  Hash,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input, Textarea } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { Tabs } from '@/components/ui/Tabs';
import { cn } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { api } from '@/lib/api';
import toast from 'react-hot-toast';

const contentTypes = [
  { id: 'text', label: 'Text', icon: <FileText size={14} /> },
  { id: 'link', label: 'Link', icon: <Link2 size={14} /> },
  { id: 'media', label: 'Media', icon: <Image size={14} /> },
];

const spaces = [
  { slug: 'crypto', name: 'crypto' },
  { slug: 'privacy', name: 'privacy' },
  { slug: 'technology', name: 'technology' },
  { slug: 'whistleblowers', name: 'whistleblowers' },
  { slug: 'defi', name: 'defi' },
];

export default function SubmitPage() {
  const router = useRouter();
  const [contentType, setContentType] = useState('text');
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [url, setUrl] = useState('');
  const [selectedSpace, setSelectedSpace] = useState('');
  const [spaceDropdown, setSpaceDropdown] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isValid = title.trim().length >= 1 && selectedSpace;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;

    setIsSubmitting(true);
    try {
      const post = await api.createPost(selectedSpace, {
        title: title.trim(),
        content: contentType === 'text' ? content : undefined,
        content_type: contentType,
        url: contentType === 'link' ? url : undefined,
      });
      toast.success('Post created!');
      router.push(`/post/${post.id}`);
    } catch (err) {
      toast.error('Failed to create post');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
        <motion.div
          initial={pageEntrance.initial}
          animate={pageEntrance.animate}
          transition={pageEntrance.transition}
        >
        <h1 className="text-2xl font-bold text-text-primary mb-6">Create a Post</h1>

        {/* Space selector */}
        <div className="relative mb-4">
          <button
            type="button"
            onClick={() => setSpaceDropdown(!spaceDropdown)}
            className={cn(
              'flex items-center gap-2 w-full max-w-xs px-3 py-2 rounded-xl',
              'bg-surface border border-surface-border',
              'text-sm transition-all duration-200',
              'hover:border-surface-hover',
              spaceDropdown && 'border-accent/30'
            )}
          >
            <Hash size={14} className="text-text-tertiary" />
            <span className={selectedSpace ? 'text-text-primary' : 'text-text-tertiary'}>
              {selectedSpace ? `s/${selectedSpace}` : 'Choose a space'}
            </span>
            <ChevronDown
              size={14}
              className={cn(
                'ml-auto text-text-tertiary transition-transform',
                spaceDropdown && 'rotate-180'
              )}
            />
          </button>

          {spaceDropdown && (
            <>
              <div className="fixed inset-0 z-10" onClick={() => setSpaceDropdown(false)} />
              <motion.div
                initial={{ opacity: 0, y: 5 }}
                animate={{ opacity: 1, y: 0 }}
                className="absolute top-full mt-1 w-full max-w-xs bg-bg-elevated rounded-xl border border-surface-border shadow-surface-lg z-20 overflow-hidden"
              >
                {spaces.map((space) => (
                  <button
                    key={space.slug}
                    onClick={() => {
                      setSelectedSpace(space.slug);
                      setSpaceDropdown(false);
                    }}
                    className={cn(
                      'flex items-center gap-2 w-full px-3 py-2 text-sm',
                      'transition-colors',
                      selectedSpace === space.slug
                        ? 'bg-accent-muted text-accent'
                        : 'text-text-secondary hover:bg-surface-hover hover:text-text-primary'
                    )}
                  >
                    <Hash size={14} />
                    s/{space.name}
                  </button>
                ))}
              </motion.div>
            </>
          )}
        </div>

        <Card className="p-6">
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Content type tabs */}
            <Tabs
              tabs={contentTypes}
              activeTab={contentType}
              onChange={setContentType}
            />

            {/* Title */}
            <Input
              label="Title"
              placeholder="An interesting title..."
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              hint="Required. 1-300 characters."
            />

            {/* Content based on type */}
            {contentType === 'text' && (
              <Textarea
                label="Body"
                placeholder="Share your thoughts... (Markdown supported)"
                value={content}
                onChange={(e) => setContent(e.target.value)}
                className="min-h-[200px]"
                hint="Optional. Supports markdown formatting."
              />
            )}

            {contentType === 'link' && (
              <Input
                label="URL"
                placeholder="https://..."
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                leftIcon={<Link2 size={16} />}
              />
            )}

            {contentType === 'media' && (
              <div className="border-2 border-dashed border-surface-border rounded-xl p-8 text-center">
                <Image size={32} className="mx-auto text-text-tertiary mb-3" />
                <p className="text-sm text-text-secondary mb-1">
                  Drop files here or click to upload
                </p>
                <p className="text-xs text-text-tertiary">
                  Images up to 10MB. EXIF data will be stripped for privacy.
                </p>
              </div>
            )}

            {/* Submit */}
            <div className="flex items-center gap-3 pt-2">
              <Button
                variant="ghost"
                onClick={() => router.back()}
                type="button"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                className="ml-auto"
                disabled={!isValid}
                isLoading={isSubmitting}
                rightIcon={<ArrowRight size={16} />}
              >
                Post
              </Button>
            </div>
          </form>
        </Card>
      </motion.div>
    </div>
  );
}
