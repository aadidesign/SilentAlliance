'use client';

import { useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { FileText, Link2, Image, ArrowRight, ArrowLeft } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input, Textarea } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { Tabs } from '@/components/ui/Tabs';
import { pageEntrance } from '@/lib/motion';
import { api } from '@/lib/api';
import toast from 'react-hot-toast';

const contentTypes = [
  { id: 'text', label: 'Text', icon: <FileText size={14} /> },
  { id: 'link', label: 'Link', icon: <Link2 size={14} /> },
  { id: 'media', label: 'Media', icon: <Image size={14} /> },
];

export default function SpaceSubmitPage() {
  const params = useParams();
  const router = useRouter();
  const slug = params.slug as string;
  const [contentType, setContentType] = useState('text');
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [url, setUrl] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const isValid = title.trim().length >= 1;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;

    setIsSubmitting(true);
    try {
      const post = await api.createPost(slug, {
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
        <button
          onClick={() => router.back()}
          className="flex items-center gap-1.5 text-sm text-text-tertiary hover:text-text-secondary transition-colors mb-4"
        >
          <ArrowLeft size={16} />
          Back to s/{slug}
        </button>

        <h1 className="text-2xl font-bold text-text-primary mb-2">
          Create Post in <span className="text-accent">s/{slug}</span>
        </h1>
        <p className="text-sm text-text-secondary mb-6">
          Share your thoughts with the community.
        </p>

        <Card className="p-6">
          <form onSubmit={handleSubmit} className="space-y-5">
            <Tabs
              tabs={contentTypes}
              activeTab={contentType}
              onChange={setContentType}
            />

            <Input
              label="Title"
              placeholder="An interesting title..."
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              hint="Required. 1-300 characters."
            />

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
                  Images up to 10MB. EXIF data will be stripped.
                </p>
              </div>
            )}

            <div className="flex items-center gap-3 pt-2">
              <Button variant="ghost" type="button" onClick={() => router.back()}>
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
