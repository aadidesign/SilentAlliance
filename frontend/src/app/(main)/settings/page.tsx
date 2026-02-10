'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  User,
  Shield,
  Key,
  Fingerprint,
  LogOut,
  Copy,
  Check,
  AlertTriangle,
  Save,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input, Textarea } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { useAuthStore } from '@/lib/store';
import { cn, shortenFingerprint } from '@/lib/utils';
import { pageEntrance } from '@/lib/motion';
import { useRequireAuth } from '@/hooks/useRequireAuth';
import { useUpdateProfile } from '@/hooks/mutations';
import toast from 'react-hot-toast';

export default function SettingsPage() {
  const router = useRouter();
  const { isAuthenticated } = useRequireAuth();
  const { identity, keypair, logout, setIdentity } = useAuthStore();
  const [displayName, setDisplayName] = useState(identity?.display_name || '');
  const [bio, setBio] = useState(identity?.bio || '');
  const [copied, setCopied] = useState<string | null>(null);
  const updateProfile = useUpdateProfile();

  const handleCopy = (value: string, label: string) => {
    navigator.clipboard.writeText(value);
    setCopied(label);
    setTimeout(() => setCopied(null), 2000);
    toast.success(`${label} copied!`);
  };

  const handleSave = () => {
    updateProfile.mutate(
      { display_name: displayName || undefined, bio: bio || undefined },
      {
        onSuccess: (updatedIdentity) => {
          setIdentity(updatedIdentity);
        },
      }
    );
  };

  if (!isAuthenticated) return null;

  const handleLogout = () => {
    logout();
    router.push('/');
    toast.success('Signed out');
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
        <motion.div
          initial={pageEntrance.initial}
          animate={pageEntrance.animate}
          transition={pageEntrance.transition}
        >
        <h1 className="text-2xl font-bold text-text-primary mb-6">Settings</h1>

        {/* Profile section */}
        <Card className="p-6 mb-4">
          <div className="flex items-center gap-2 mb-5">
            <User size={18} className="text-accent" />
            <h2 className="text-base font-semibold text-text-primary">Profile</h2>
          </div>

          <div className="space-y-4">
            <Input
              label="Display Name"
              placeholder="Anonymous"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              hint="1-50 characters. Leave empty to stay anonymous."
            />

            <Textarea
              label="Bio"
              placeholder="Tell others about yourself..."
              value={bio}
              onChange={(e) => setBio(e.target.value)}
              hint="Optional. Up to 500 characters."
              className="min-h-[100px]"
            />

            <div className="flex justify-end">
              <Button
                size="sm"
                onClick={handleSave}
                leftIcon={<Save size={14} />}
                isLoading={updateProfile.isPending}
              >
                Save Changes
              </Button>
            </div>
          </div>
        </Card>

        {/* Identity section */}
        <Card className="p-6 mb-4">
          <div className="flex items-center gap-2 mb-5">
            <Shield size={18} className="text-accent" />
            <h2 className="text-base font-semibold text-text-primary">Identity</h2>
          </div>

          <div className="space-y-4">
            {/* Fingerprint */}
            <div>
              <label className="text-sm font-medium text-text-secondary mb-1.5 block">
                Public Key Fingerprint
              </label>
              <div className="flex items-center gap-2">
                <div className="flex-1 h-10 px-3 bg-bg-tertiary border border-surface-border rounded-xl flex items-center">
                  <Fingerprint size={14} className="text-text-tertiary mr-2" />
                  <span className="text-xs font-mono text-text-secondary truncate">
                    {identity?.public_key_fingerprint || 'Not available'}
                  </span>
                </div>
                <Button
                  variant="secondary"
                  size="icon"
                  onClick={() =>
                    identity?.public_key_fingerprint &&
                    handleCopy(identity.public_key_fingerprint, 'Fingerprint')
                  }
                >
                  {copied === 'Fingerprint' ? <Check size={14} /> : <Copy size={14} />}
                </Button>
              </div>
            </div>

            {/* Public key */}
            {keypair && (
              <div>
                <label className="text-sm font-medium text-text-secondary mb-1.5 block">
                  Public Key
                </label>
                <div className="flex items-center gap-2">
                  <div className="flex-1 h-10 px-3 bg-bg-tertiary border border-surface-border rounded-xl flex items-center">
                    <Key size={14} className="text-text-tertiary mr-2" />
                    <span className="text-xs font-mono text-text-secondary truncate">
                      {keypair.publicKey}
                    </span>
                  </div>
                  <Button
                    variant="secondary"
                    size="icon"
                    onClick={() => handleCopy(keypair.publicKey, 'Public key')}
                  >
                    {copied === 'Public key' ? <Check size={14} /> : <Copy size={14} />}
                  </Button>
                </div>
              </div>
            )}
          </div>
        </Card>

        {/* Security section */}
        <Card className="p-6 mb-4">
          <div className="flex items-center gap-2 mb-5">
            <Key size={18} className="text-accent" />
            <h2 className="text-base font-semibold text-text-primary">Security</h2>
          </div>

          <div className="bg-warning-muted/30 rounded-xl border border-warning/20 p-4 mb-4">
            <div className="flex items-start gap-3">
              <AlertTriangle size={18} className="text-warning shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-medium text-warning mb-1">
                  Backup your secret key
                </p>
                <p className="text-xs text-text-secondary">
                  Your secret key is the only way to access your identity. If you lose it,
                  there is no recovery process.
                </p>
              </div>
            </div>
          </div>

          {keypair && (
            <Button
              variant="outline"
              onClick={() => handleCopy(keypair.secretKey, 'Secret key')}
              leftIcon={copied === 'Secret key' ? <Check size={14} /> : <Copy size={14} />}
            >
              {copied === 'Secret key' ? 'Copied!' : 'Copy Secret Key'}
            </Button>
          )}
        </Card>

        {/* Danger zone */}
        <Card className="p-6 border-danger/20">
          <div className="flex items-center gap-2 mb-5">
            <AlertTriangle size={18} className="text-danger" />
            <h2 className="text-base font-semibold text-danger">Danger Zone</h2>
          </div>

          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-text-primary font-medium">Sign Out</p>
              <p className="text-xs text-text-tertiary">
                This will clear your session on this device.
              </p>
            </div>
            <Button
              variant="danger"
              size="sm"
              onClick={handleLogout}
              leftIcon={<LogOut size={14} />}
            >
              Sign Out
            </Button>
          </div>
        </Card>
      </motion.div>
    </div>
  );
}
