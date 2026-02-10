'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { motion } from 'framer-motion';
import { Shield, KeyRound, ArrowRight, Github, MessageCircle } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Textarea } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { useAuthStore } from '@/lib/store';
import { api } from '@/lib/api';
import { getFingerprint, signChallenge } from '@/lib/crypto';
import toast from 'react-hot-toast';

export default function LoginPage() {
  const router = useRouter();
  const { setTokens, setKeypair, setIdentity } = useAuthStore();
  const [secretKey, setSecretKey] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  const handleLogin = async () => {
    if (!secretKey.trim()) {
      toast.error('Please enter your secret key');
      return;
    }

    setIsLoading(true);
    try {
      // Derive public key and fingerprint from secret key
      const nacl = (await import('tweetnacl')).default;
      const naclUtil = (await import('tweetnacl-util')).default;

      const secretKeyBytes = naclUtil.decodeBase64(secretKey.trim());
      const keypairFromSecret = nacl.sign.keyPair.fromSecretKey(secretKeyBytes);
      const publicKeyBase64 = naclUtil.encodeBase64(keypairFromSecret.publicKey);
      const fingerprint = await getFingerprint(publicKeyBase64);

      // Get challenge
      const { challenge } = await api.getChallenge(fingerprint);

      // Sign challenge
      const signature = signChallenge(challenge, secretKey.trim());

      // Login
      const tokens = await api.login(fingerprint, challenge, signature);
      setTokens(tokens);
      setKeypair({ publicKey: publicKeyBase64, secretKey: secretKey.trim() });

      // Fetch identity
      const identity = await api.getMe();
      setIdentity(identity);

      toast.success('Welcome back!');
      router.push('/feed');
    } catch (err) {
      toast.error('Login failed. Check your secret key and try again.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-bg-primary flex items-center justify-center px-4 py-12">
      {/* Background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 right-1/3 w-[500px] h-[500px] bg-accent/8 rounded-full blur-[150px]" />
        <div className="absolute bottom-1/4 left-1/3 w-[400px] h-[400px] bg-cyan-500/6 rounded-full blur-[120px]" />
      </div>

      <div className="relative w-full max-w-lg">
        {/* Logo */}
        <div className="text-center mb-8">
          <Link href="/" className="inline-flex items-center gap-2.5">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent to-accent-secondary flex items-center justify-center">
              <Shield size={22} className="text-text-on-accent" />
            </div>
            <span className="font-bold text-xl">
              Silent<span className="gradient-text">Alliance</span>
            </span>
          </Link>
        </div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          <Card className="p-8">
            <div className="space-y-6">
              <div className="text-center">
                <div className="w-16 h-16 rounded-2xl bg-accent-muted flex items-center justify-center mx-auto mb-4">
                  <KeyRound size={28} className="text-accent" />
                </div>
                <h1 className="text-2xl font-bold mb-2">Welcome Back</h1>
                <p className="text-sm text-text-secondary">
                  Enter your secret key to access your identity
                </p>
              </div>

              {/* Secret key input */}
              <Textarea
                label="Secret Key"
                placeholder="Paste your base64-encoded Ed25519 secret key..."
                value={secretKey}
                onChange={(e) => setSecretKey(e.target.value)}
                className="font-mono text-xs min-h-[80px]"
                hint="Your key is only used locally for signing. It's never sent to the server."
              />

              <Button
                className="w-full"
                size="lg"
                onClick={handleLogin}
                isLoading={isLoading}
                rightIcon={<ArrowRight size={18} />}
              >
                Sign In
              </Button>

              {/* OAuth divider */}
              <div className="flex items-center gap-3">
                <div className="flex-1 border-t border-surface-border" />
                <span className="text-xs text-text-tertiary">or continue with</span>
                <div className="flex-1 border-t border-surface-border" />
              </div>

              {/* OAuth buttons */}
              <div className="grid grid-cols-2 gap-3">
                <Button
                  variant="secondary"
                  leftIcon={<Github size={16} />}
                  onClick={() => {
                    window.location.href = `${process.env.NEXT_PUBLIC_API_URL || ''}/api/v1/auth/oauth/authorize?provider=github`;
                  }}
                >
                  GitHub
                </Button>
                <Button
                  variant="secondary"
                  leftIcon={<MessageCircle size={16} />}
                  onClick={() => {
                    window.location.href = `${process.env.NEXT_PUBLIC_API_URL || ''}/api/v1/auth/oauth/authorize?provider=discord`;
                  }}
                >
                  Discord
                </Button>
              </div>
            </div>
          </Card>
        </motion.div>

        <p className="text-center text-sm text-text-tertiary mt-6">
          Don&apos;t have an identity?{' '}
          <Link href="/register" className="text-accent hover:text-accent-hover transition-colors">
            Create one
          </Link>
        </p>
      </div>
    </div>
  );
}
