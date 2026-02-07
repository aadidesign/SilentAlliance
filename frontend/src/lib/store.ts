import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { AuthTokens, Identity } from '@/types';

// ==================== Auth Store ====================

interface AuthState {
  tokens: AuthTokens | null;
  identity: Identity | null;
  keypair: { publicKey: string; secretKey: string } | null;
  isAuthenticated: boolean;
  setTokens: (tokens: AuthTokens) => void;
  setIdentity: (identity: Identity) => void;
  setKeypair: (keypair: { publicKey: string; secretKey: string }) => void;
  logout: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      tokens: null,
      identity: null,
      keypair: null,
      isAuthenticated: false,
      setTokens: (tokens) => set({ tokens, isAuthenticated: true }),
      setIdentity: (identity) => set({ identity }),
      setKeypair: (keypair) => set({ keypair }),
      logout: () => set({ tokens: null, identity: null, keypair: null, isAuthenticated: false }),
    }),
    {
      name: 'sa-auth',
      partialize: (state) => ({
        tokens: state.tokens,
        keypair: state.keypair,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);

// ==================== UI Store ====================

interface UIState {
  sidebarOpen: boolean;
  mobileMenuOpen: boolean;
  createPostModalOpen: boolean;
  toggleSidebar: () => void;
  toggleMobileMenu: () => void;
  setCreatePostModalOpen: (open: boolean) => void;
  closeMobileMenu: () => void;
}

export const useUIStore = create<UIState>()((set) => ({
  sidebarOpen: true,
  mobileMenuOpen: false,
  createPostModalOpen: false,
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  toggleMobileMenu: () => set((s) => ({ mobileMenuOpen: !s.mobileMenuOpen })),
  setCreatePostModalOpen: (open) => set({ createPostModalOpen: open }),
  closeMobileMenu: () => set({ mobileMenuOpen: false }),
}));
