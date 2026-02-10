'use client';

import { useState, useCallback } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { motion } from 'framer-motion';
import {
  Shield,
  KeyRound,
  Copy,
  Check,
  AlertTriangle,
  ArrowRight,
  Fingerprint,
  Lock,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Card } from '@/components/ui/Card';
import { cn } from '@/lib/utils';
import { generateKeypair, type KeyPair } from '@/lib/crypto';
import { useAuthStore } from '@/lib/store';
import { api } from '@/lib/api';
import toast from 'react-hot-toast';

type Step = 'intro' | 'generate' | 'backup' | 'name' | 'complete';

export default function RegisterPage() {
  const router = useRouter();
  const { setTokens, setKeypair, setIdentity } = useAuthStore();
  const [step, setStep] = useState<Step>('intro');
  const [keypair, setLocalKeypair] = useState<KeyPair | null>(null);
  const [displayName, setDisplayName] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [isRegistering, setIsRegistering] = useState(false);
  const [copied, setCopied] = useState(false);
  const [backedUp, setBackedUp] = useState(false);

  const handleGenerate = useCallback(async () => {
    setIsGenerating(true);
    try {
      // Add slight delay for visual feedback
      await new Promise((r) => setTimeout(r, 800));
      const kp = await generateKeypair();
      setLocalKeypair(kp);
      setStep('backup');
    } catch (err) {
      toast.error('Failed to generate keypair');
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const handleCopyKey = async () => {
    if (!keypair) return;
    await navigator.clipboard.writeText(keypair.secretKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleRegister = async () => {
    if (!keypair) return;
    setIsRegistering(true);
    try {
      const tokens = await api.register(keypair.publicKey, displayName || undefined);
      setTokens(tokens);
      setKeypair({ publicKey: keypair.publicKey, secretKey: keypair.secretKey });

      // Fetch identity
      const identity = await api.getMe();
      setIdentity(identity);

      setStep('complete');
      toast.success('Identity created!');

      setTimeout(() => router.push('/feed'), 1500);
    } catch (err) {
      toast.error('Registration failed. Please try again.');
    } finally {
      setIsRegistering(false);
    }
  };

  return (
    <div className="min-h-screen bg-bg-primary flex items-center justify-center px-4 py-12">
      {/* Background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 left-1/3 w-[500px] h-[500px] bg-accent/8 rounded-full blur-[150px]" />
        <div className="absolute bottom-1/4 right-1/3 w-[400px] h-[400px] bg-cyan-500/6 rounded-full blur-[120px]" />
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
          key={step}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          {/* Step: Intro */}
          {step === 'intro' && (
            <Card className="p-8">
              <div className="text-center space-y-4">
                <div className="w-16 h-16 rounded-2xl bg-accent-muted flex items-center justify-center mx-auto">
                  <KeyRound size={28} className="text-accent" />
                </div>
                <h1 className="text-2xl font-bold">Create Your Identity</h1>
                <p className="text-text-secondary text-sm leading-relaxed max-w-sm mx-auto">
                  Your identity is a cryptographic keypair generated in your browser.
                  No email, no phone, no personal data.
                </p>

                <div className="space-y-3 text-left py-4">
                  {[
                    {
                      icon: <Fingerprint size={16} />,
                      text: 'Ed25519 keypair generated locally',
                    },
                    {
                      icon: <Lock size={16} />,
                      text: 'Private key never leaves your device',
                    },
                    {
                      icon: <Shield size={16} />,
                      text: 'Challenge-response authentication',
                    },
                  ].map((item) => (
                    <div
                      key={item.text}
                      className="flex items-center gap-3 text-sm text-text-secondary"
                    >
                      <span className="text-accent">{item.icon}</span>
                      {item.text}
                    </div>
                  ))}
                </div>

                <Button
                  className="w-full"
                  size="lg"
                  onClick={() => {
                    setStep('generate');
                    handleGenerate();
                  }}
                  rightIcon={<ArrowRight size={18} />}
                >
                  Generate Keypair
                </Button>
              </div>
            </Card>
          )}

          {/* Step: Generating */}
          {step === 'generate' && (
            <Card className="p-8">
              <div className="text-center space-y-6">
                <div className="w-16 h-16 rounded-2xl bg-accent-muted flex items-center justify-center mx-auto">
                  <motion.div
                    animate={{ rotate: 360 }}
                    transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
                  >
                    <KeyRound size={28} className="text-accent" />
                  </motion.div>
                </div>
                <h2 className="text-xl font-bold">Generating Keypair...</h2>
                <p className="text-sm text-text-secondary">
                  Creating your Ed25519 identity locally
                </p>
                <div className="flex justify-center">
                  <div className="flex gap-1">
                    {[0, 1, 2].map((i) => (
                      <motion.div
                        key={i}
                        className="w-2 h-2 bg-accent rounded-full"
                        animate={{ opacity: [0.3, 1, 0.3] }}
                        transition={{ duration: 1, repeat: Infinity, delay: i * 0.2 }}
                      />
                    ))}
                  </div>
                </div>
              </div>
            </Card>
          )}

          {/* Step: Backup */}
          {step === 'backup' && keypair && (
            <Card className="p-8">
              <div className="space-y-6">
                <div className="text-center">
                  <div className="w-16 h-16 rounded-2xl bg-warning-muted flex items-center justify-center mx-auto mb-4">
                    <AlertTriangle size={28} className="text-warning" />
                  </div>
                  <h2 className="text-xl font-bold mb-2">Backup Your Secret Key</h2>
                  <p className="text-sm text-text-secondary">
                    This is your only way to recover your identity. Save it securely.
                  </p>
                </div>

                {/* Key display */}
                <div className="bg-bg-secondary rounded-xl border border-surface-border p-4 space-y-3">
                  <div>
                    <p className="text-xs text-text-tertiary mb-1">Public Key (Fingerprint)</p>
                    <p className="text-xs font-mono text-text-secondary break-all">
                      {keypair.fingerprint}
                    </p>
                  </div>
                  <div className="border-t border-surface-border" />
                  <div>
                    <p className="text-xs text-text-tertiary mb-1">Secret Key (SAVE THIS)</p>
                    <p className="text-xs font-mono text-accent break-all">
                      {keypair.secretKey}
                    </p>
                  </div>
                </div>

                {/* Copy button */}
                <Button
                  variant="outline"
                  className="w-full"
                  onClick={handleCopyKey}
                  leftIcon={copied ? <Check size={16} /> : <Copy size={16} />}
                >
                  {copied ? 'Copied!' : 'Copy Secret Key'}
                </Button>

                {/* Backup confirmation */}
                <label className="flex items-start gap-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={backedUp}
                    onChange={(e) => setBackedUp(e.target.checked)}
                    className="mt-0.5 rounded border-surface-border bg-bg-tertiary text-accent focus:ring-accent/20"
                  />
                  <span className="text-sm text-text-secondary">
                    I have securely saved my secret key. I understand it cannot be recovered if lost.
                  </span>
                </label>

                <Button
                  className="w-full"
                  size="lg"
                  disabled={!backedUp}
                  onClick={() => setStep('name')}
                  rightIcon={<ArrowRight size={18} />}
                >
                  Continue
                </Button>
              </div>
            </Card>
          )}

          {/* Step: Display Name */}
          {step === 'name' && (
            <Card className="p-8">
              <div className="space-y-6">
                <div className="text-center">
                  <h2 className="text-xl font-bold mb-2">Choose a Display Name</h2>
                  <p className="text-sm text-text-secondary">
                    Optional. You can always change this later or stay anonymous.
                  </p>
                </div>

                <Input
                  placeholder="Anonymous"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  hint="1-50 characters. Leave empty to stay anonymous."
                />

                <div className="flex gap-3">
                  <Button
                    variant="secondary"
                    className="flex-1"
                    onClick={() => {
                      setDisplayName('');
                      handleRegister();
                    }}
                    isLoading={isRegistering}
                  >
                    Skip
                  </Button>
                  <Button
                    className="flex-1"
                    onClick={handleRegister}
                    isLoading={isRegistering}
                    rightIcon={<ArrowRight size={18} />}
                  >
                    Create Identity
                  </Button>
                </div>
              </div>
            </Card>
          )}

          {/* Step: Complete */}
          {step === 'complete' && (
            <Card className="p-8">
              <div className="text-center space-y-4">
                <motion.div
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ type: 'spring', damping: 10, stiffness: 100 }}
                  className="w-16 h-16 rounded-2xl bg-success-muted flex items-center justify-center mx-auto"
                >
                  <Check size={32} className="text-success" />
                </motion.div>
                <h2 className="text-xl font-bold">Welcome to the Alliance</h2>
                <p className="text-sm text-text-secondary">
                  Your anonymous identity has been created. Redirecting...
                </p>
              </div>
            </Card>
          )}
        </motion.div>

        {/* Login link */}
        {step !== 'complete' && (
          <p className="text-center text-sm text-text-tertiary mt-6">
            Already have a keypair?{' '}
            <Link href="/login" className="text-accent hover:text-accent-hover transition-colors">
              Log in
            </Link>
          </p>
        )}
      </div>
    </div>
  );
}
