'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Hash, Lock, Globe, AlertCircle, ArrowRight } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input, Textarea } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { cn } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { api } from '@/lib/api';
import toast from 'react-hot-toast';

export default function CreateSpacePage() {
  const router = useRouter();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [isPrivate, setIsPrivate] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const slug = name.toLowerCase().replace(/[^a-z0-9_]/g, '_').replace(/_+/g, '_');
  const isValid = name.length >= 3 && name.length <= 50 && /^[a-zA-Z0-9_]+$/.test(name);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!isValid) return;

    setIsSubmitting(true);
    try {
      const space = await api.createSpace({
        name,
        description: description || undefined,
        is_private: isPrivate,
      });
      toast.success('Space created!');
      router.push(`/s/${space.slug}`);
    } catch (err) {
      toast.error('Failed to create space');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="max-w-xl mx-auto">
        <motion.div
          initial={pageEntrance.initial}
          animate={pageEntrance.animate}
          transition={pageEntrance.transition}
        >
        <h1 className="text-2xl font-bold text-text-primary mb-2">Create a Space</h1>
        <p className="text-sm text-text-secondary mb-6">
          Start a community around a topic you care about.
        </p>

        <Card className="p-6">
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Name */}
            <div>
              <Input
                label="Space Name"
                placeholder="e.g. privacy_tools"
                value={name}
                onChange={(e) => setName(e.target.value)}
                leftIcon={<Hash size={16} />}
                hint="3-50 characters. Letters, numbers, and underscores only."
                error={name.length > 0 && !isValid ? 'Invalid name format' : undefined}
              />
              {name && isValid && (
                <p className="text-xs text-text-tertiary mt-1.5">
                  Your space will be at <span className="text-accent font-mono">s/{slug}</span>
                </p>
              )}
            </div>

            {/* Description */}
            <Textarea
              label="Description"
              placeholder="What is this space about?"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              hint="Optional. Up to 500 characters."
            />

            {/* Visibility */}
            <div>
              <p className="text-sm font-medium text-text-secondary mb-3">Visibility</p>
              <div className="grid grid-cols-2 gap-3">
                <button
                  type="button"
                  onClick={() => setIsPrivate(false)}
                  className={cn(
                    'flex items-center gap-3 p-3 rounded-xl border transition-all duration-200',
                    !isPrivate
                      ? 'bg-accent-muted border-accent/30 text-accent'
                      : 'bg-bg-tertiary border-surface-border text-text-secondary hover:border-surface-hover'
                  )}
                >
                  <Globe size={18} />
                  <div className="text-left">
                    <p className="text-sm font-medium">Public</p>
                    <p className="text-2xs opacity-70">Anyone can view and join</p>
                  </div>
                </button>
                <button
                  type="button"
                  onClick={() => setIsPrivate(true)}
                  className={cn(
                    'flex items-center gap-3 p-3 rounded-xl border transition-all duration-200',
                    isPrivate
                      ? 'bg-accent-muted border-accent/30 text-accent'
                      : 'bg-bg-tertiary border-surface-border text-text-secondary hover:border-surface-hover'
                  )}
                >
                  <Lock size={18} />
                  <div className="text-left">
                    <p className="text-sm font-medium">Private</p>
                    <p className="text-2xs opacity-70">Invite-only access</p>
                  </div>
                </button>
              </div>
            </div>

            {/* Submit */}
            <Button
              type="submit"
              className="w-full"
              size="lg"
              disabled={!isValid}
              isLoading={isSubmitting}
              rightIcon={<ArrowRight size={18} />}
            >
              Create Space
            </Button>
          </form>
        </Card>
      </motion.div>
    </div>
  );
}
