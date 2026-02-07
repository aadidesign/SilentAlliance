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
    <html lang="en" className="dark">
      <body className="min-h-screen bg-bg-primary text-text-primary antialiased">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
