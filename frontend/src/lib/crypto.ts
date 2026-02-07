/**
 * Cryptographic utilities for SilentAlliance
 *
 * Uses tweetnacl for Ed25519 keypair generation and signing.
 * In production, this would also handle X25519 key exchange for E2E messaging.
 */
import nacl from 'tweetnacl';
import naclUtil from 'tweetnacl-util';

export interface KeyPair {
  publicKey: string; // Base64
  secretKey: string; // Base64
  fingerprint: string; // SHA-256 hex of public key
}

/**
 * Generate a new Ed25519 keypair for pseudonymous identity
 */
export async function generateKeypair(): Promise<KeyPair> {
  const keypair = nacl.sign.keyPair();

  const publicKeyBase64 = naclUtil.encodeBase64(keypair.publicKey);
  const secretKeyBase64 = naclUtil.encodeBase64(keypair.secretKey);

  // Create fingerprint (SHA-256 of public key)
  const hashBuffer = await crypto.subtle.digest('SHA-256', keypair.publicKey.buffer as ArrayBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const fingerprint = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return {
    publicKey: publicKeyBase64,
    secretKey: secretKeyBase64,
    fingerprint,
  };
}

/**
 * Sign a challenge with the secret key
 */
export function signChallenge(challenge: string, secretKeyBase64: string): string {
  const secretKey = naclUtil.decodeBase64(secretKeyBase64);
  const messageBytes = naclUtil.decodeUTF8(challenge);
  const signature = nacl.sign.detached(messageBytes, secretKey);
  return naclUtil.encodeBase64(signature);
}

/**
 * Get fingerprint from a public key
 */
export async function getFingerprint(publicKeyBase64: string): Promise<string> {
  const publicKey = naclUtil.decodeBase64(publicKeyBase64);
  const hashBuffer = await crypto.subtle.digest('SHA-256', publicKey.buffer as ArrayBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}
