import type { Metadata } from 'next';
import { Providers } from './providers';
import './globals.css';

export const metadata: Metadata = {
  title: 'SilentAlliance — Anonymous Discussion Platform',
  description:
    'A privacy-first anonymous social discussion platform. No email, no phone, no tracking. Just conversations.',
  keywords: ['anonymous', 'privacy', 'social', 'discussion', 'encrypted', 'decentralized'],
  openGraph: {
    title: 'SilentAlliance',
    description: 'Privacy-first anonymous social discussion platform',
    type: 'website',
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body className="min-h-screen bg-bg-primary text-text-primary antialiased" suppressHydrationWarning>
        <a
          href="#main-content"
          className="sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[100] focus:px-4 focus:py-2 focus:rounded-xl focus:bg-accent focus:text-text-on-accent focus:text-sm focus:font-medium focus:shadow-glow"
        >
          Skip to content
        </a>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
