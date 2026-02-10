import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuthStore } from '@/lib/store';

/**
 * Redirects unauthenticated users to /login.
 * Call at the top of any page that requires authentication.
 * Returns { isAuthenticated, identity, tokens } for convenience.
 */
export function useRequireAuth() {
  const router = useRouter();
  const { isAuthenticated, identity, tokens } = useAuthStore();

  useEffect(() => {
    if (!isAuthenticated) {
      router.replace('/login');
    }
  }, [isAuthenticated, router]);

  return { isAuthenticated, identity, tokens };
}
