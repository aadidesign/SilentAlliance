'use client';

import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  Shield,
  Lock,
  Eye,
  MessageSquare,
  Users,
  Zap,
  ArrowRight,
  Fingerprint,
  KeyRound,
  Globe,
  ChevronRight,
} from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/utils';

const fadeUp = {
  initial: { opacity: 0, y: 30 },
  animate: { opacity: 1, y: 0 },
};

const stagger = {
  animate: {
    transition: { staggerChildren: 0.1 },
  },
};

export default function LandingPage() {
  const router = useRouter();

  return (
    <div className="min-h-screen bg-bg-primary overflow-hidden">
      {/* Navigation */}
      <nav className="fixed top-0 left-0 right-0 z-50 glass-strong">
        <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-accent-secondary flex items-center justify-center">
              <Shield size={18} className="text-text-on-accent" />
            </div>
            <span className="font-bold text-lg">
              Silent<span className="gradient-text">Alliance</span>
            </span>
          </div>
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="sm" onClick={() => router.push('/login')}>
              Log in
            </Button>
            <Button variant="primary" size="sm" onClick={() => router.push('/register')}>
              Get Started
            </Button>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="relative pt-32 pb-20 px-6">
        {/* Background effects */}
        <div className="absolute inset-0 overflow-hidden">
          {/* Gradient orbs */}
          <div className="absolute top-20 left-1/4 w-[500px] h-[500px] bg-accent/10 rounded-full blur-[120px] animate-float" />
          <div className="absolute top-40 right-1/4 w-[400px] h-[400px] bg-cyan-500/8 rounded-full blur-[100px] animate-float" style={{ animationDelay: '2s' }} />
          <div className="absolute bottom-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] bg-accent/5 rounded-full blur-[120px]" />
          {/* Grid pattern */}
          <div
            className="absolute inset-0 opacity-[0.03]"
            style={{
              backgroundImage: `linear-gradient(var(--text-tertiary) 1px, transparent 1px), linear-gradient(90deg, var(--text-tertiary) 1px, transparent 1px)`,
              backgroundSize: '60px 60px',
            }}
          />
        </div>

        <motion.div
          className="relative max-w-4xl mx-auto text-center"
          initial="initial"
          animate="animate"
          variants={stagger}
        >
          {/* Badge */}
          <motion.div variants={fadeUp} transition={{ duration: 0.6 }}>
            <div className="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-accent-muted border border-accent/20 mb-8">
              <div className="w-1.5 h-1.5 rounded-full bg-accent animate-pulse-glow" />
              <span className="text-xs font-medium text-accent">Privacy-First Platform</span>
            </div>
          </motion.div>

          {/* Headline */}
          <motion.h1
            variants={fadeUp}
            transition={{ duration: 0.6, delay: 0.1 }}
            className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight leading-[1.1] mb-6"
          >
            Speak Freely.
            <br />
            <span className="gradient-text">Stay Anonymous.</span>
          </motion.h1>

          {/* Subheadline */}
          <motion.p
            variants={fadeUp}
            transition={{ duration: 0.6, delay: 0.2 }}
            className="text-lg sm:text-xl text-text-secondary max-w-2xl mx-auto mb-10 text-balance"
          >
            Join the conversation without sacrificing your privacy. No email, no phone number,
            no tracking. Just your ideas, protected by military-grade encryption.
          </motion.p>

          {/* CTA buttons */}
          <motion.div
            variants={fadeUp}
            transition={{ duration: 0.6, delay: 0.3 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-4"
          >
            <Button
              size="lg"
              onClick={() => router.push('/register')}
              rightIcon={<ArrowRight size={18} />}
              className="w-full sm:w-auto px-8"
            >
              Create Anonymous Identity
            </Button>
            <Button
              variant="outline"
              size="lg"
              onClick={() => router.push('/all')}
              className="w-full sm:w-auto px-8"
            >
              Explore Spaces
            </Button>
          </motion.div>

          {/* Stats */}
          <motion.div
            variants={fadeUp}
            transition={{ duration: 0.6, delay: 0.4 }}
            className="flex items-center justify-center gap-8 sm:gap-12 mt-16"
          >
            {[
              { value: 'E2E', label: 'Encrypted Messages' },
              { value: 'Zero', label: 'Data Collection' },
              { value: 'Ed25519', label: 'Identity System' },
            ].map((stat) => (
              <div key={stat.label} className="text-center">
                <p className="text-2xl sm:text-3xl font-bold gradient-text">{stat.value}</p>
                <p className="text-xs sm:text-sm text-text-tertiary mt-1">{stat.label}</p>
              </div>
            ))}
          </motion.div>
        </motion.div>
      </section>

      {/* Features Section */}
      <section className="relative py-24 px-6">
        <div className="max-w-6xl mx-auto">
          <motion.div
            className="text-center mb-16"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
          >
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Built for <span className="text-accent">Privacy</span>
            </h2>
            <p className="text-text-secondary max-w-lg mx-auto">
              Every feature designed with one principle: your privacy is non-negotiable.
            </p>
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {[
              {
                icon: <Fingerprint size={24} />,
                title: 'Pseudonymous Identity',
                description:
                  'Your identity is a cryptographic keypair. No email, no phone, no personal data required.',
                gradient: 'from-violet-500/20 to-purple-500/20',
              },
              {
                icon: <Lock size={24} />,
                title: 'End-to-End Encryption',
                description:
                  'Messages encrypted with X25519 + ChaCha20-Poly1305. The server never sees your content.',
                gradient: 'from-cyan-500/20 to-blue-500/20',
              },
              {
                icon: <Eye size={24} />,
                title: 'Zero Knowledge',
                description:
                  'We strip EXIF data from uploads, don\'t track IPs, and store the minimum data possible.',
                gradient: 'from-emerald-500/20 to-teal-500/20',
              },
              {
                icon: <Users size={24} />,
                title: 'Community Spaces',
                description:
                  'Create and join communities around topics you care about. Public or private.',
                gradient: 'from-orange-500/20 to-amber-500/20',
              },
              {
                icon: <MessageSquare size={24} />,
                title: 'Threaded Discussions',
                description:
                  'Rich markdown posts with threaded comments. Vote on content. Shape the conversation.',
                gradient: 'from-pink-500/20 to-rose-500/20',
              },
              {
                icon: <Zap size={24} />,
                title: 'Real-time Updates',
                description:
                  'WebSocket-powered live notifications. Never miss a reply or mention.',
                gradient: 'from-yellow-500/20 to-orange-500/20',
              },
            ].map((feature, i) => (
              <motion.div
                key={feature.title}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: i * 0.1 }}
              >
                <div className="group relative h-full bg-surface rounded-2xl border border-surface-border p-6 hover:border-accent/20 transition-all duration-300">
                  {/* Icon */}
                  <div
                    className={cn(
                      'w-12 h-12 rounded-xl bg-gradient-to-br flex items-center justify-center mb-4',
                      'text-text-primary',
                      feature.gradient
                    )}
                  >
                    {feature.icon}
                  </div>
                  <h3 className="text-lg font-semibold text-text-primary mb-2">
                    {feature.title}
                  </h3>
                  <p className="text-sm text-text-secondary leading-relaxed">
                    {feature.description}
                  </p>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* How it works section */}
      <section className="relative py-24 px-6">
        <div className="max-w-4xl mx-auto">
          <motion.div
            className="text-center mb-16"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
          >
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              How It <span className="text-accent">Works</span>
            </h2>
            <p className="text-text-secondary max-w-lg mx-auto">
              Get started in seconds. No sign-up forms. No verification emails.
            </p>
          </motion.div>

          <div className="space-y-6">
            {[
              {
                step: '01',
                icon: <KeyRound size={22} />,
                title: 'Generate Your Keys',
                description:
                  'A unique Ed25519 keypair is generated in your browser. Your private key never leaves your device.',
              },
              {
                step: '02',
                icon: <Shield size={22} />,
                title: 'Create Your Identity',
                description:
                  'Pick a display name (or stay anonymous). Your identity is tied to your public key, not personal info.',
              },
              {
                step: '03',
                icon: <Globe size={22} />,
                title: 'Join the Alliance',
                description:
                  'Explore spaces, share ideas, vote on content, and send encrypted messages. All without a trace.',
              },
            ].map((item, i) => (
              <motion.div
                key={item.step}
                initial={{ opacity: 0, x: -20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.5, delay: i * 0.15 }}
                className="flex gap-6 items-start"
              >
                <div className="shrink-0 w-12 h-12 rounded-xl bg-accent-muted flex items-center justify-center text-accent font-mono font-bold text-sm">
                  {item.step}
                </div>
                <div className="flex-1 bg-surface rounded-2xl border border-surface-border p-5">
                  <div className="flex items-center gap-3 mb-2">
                    <span className="text-accent">{item.icon}</span>
                    <h3 className="text-lg font-semibold text-text-primary">{item.title}</h3>
                  </div>
                  <p className="text-sm text-text-secondary leading-relaxed">
                    {item.description}
                  </p>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative py-24 px-6">
        <div className="max-w-3xl mx-auto text-center">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
            className="relative bg-surface rounded-3xl border border-surface-border p-12 overflow-hidden"
          >
            {/* Background glow */}
            <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[400px] h-[200px] bg-accent/10 rounded-full blur-[80px]" />

            <div className="relative">
              <h2 className="text-3xl sm:text-4xl font-bold mb-4">
                Ready to Join the
                <br />
                <span className="text-accent">Silent Alliance?</span>
              </h2>
              <p className="text-text-secondary mb-8 max-w-md mx-auto">
                Your voice matters. Your identity doesn&apos;t have to be exposed for it to be heard.
              </p>
              <Button
                size="lg"
                onClick={() => router.push('/register')}
                rightIcon={<ChevronRight size={18} />}
                className="px-10"
              >
                Generate Your Identity
              </Button>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-surface-border py-8 px-6">
        <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <Shield size={16} className="text-accent" />
            <span className="text-sm text-text-tertiary">
              SilentAlliance &mdash; No tracking. No data harvesting. Just conversations.
            </span>
          </div>
          <div className="flex items-center gap-6">
            <a href="#" className="text-sm text-text-tertiary hover:text-text-secondary transition-colors">
              Privacy
            </a>
            <a href="#" className="text-sm text-text-tertiary hover:text-text-secondary transition-colors">
              Terms
            </a>
            <a href="#" className="text-sm text-text-tertiary hover:text-text-secondary transition-colors">
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}
