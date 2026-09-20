// DiskAnalyzer licensing server — Ed25519 keypair (dev).
//
// Per 13-LICENSING-SERVER.md §13.5:
//   - The dev keypair is committed (it's dev-only).
//   - The prod keypair lives in a secret store; never committed.
//
// This file generates a fresh dev keypair on first run if none is present,
// then loads it on subsequent runs. The private key signs entitlements; the
// matching public key is embedded in the client (crates/napi + packages/license-client).

import {
  type KeyObject,
  createPrivateKey,
  createPublicKey,
  generateKeyPairSync,
} from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const KEYS_DIR = join(__dirname, 'keys');
const PRIVATE_KEY_PATH = join(KEYS_DIR, 'dev.private.pem');
const PUBLIC_KEY_DER_PATH = join(KEYS_DIR, 'dev.public.der');

export interface DevKeyPair {
  privateKey: KeyObject;
  publicKey: KeyObject;
  publicKeyDer: Buffer;
}

/// Load or generate the dev Ed25519 keypair.
export function loadOrGenerateDevKeys(): DevKeyPair {
  if (!existsSync(PRIVATE_KEY_PATH)) {
    // Generate a fresh Ed25519 keypair.
    const { privateKey, publicKey } = generateKeyPairSync('ed25519');
    mkdirSync(KEYS_DIR, { recursive: true });
    writeFileSync(PRIVATE_KEY_PATH, privateKey.export({ type: 'pkcs8', format: 'pem' }), {
      mode: 0o600,
    });
    const der = publicKey.export({ type: 'spki', format: 'der' }) as Buffer;
    writeFileSync(PUBLIC_KEY_DER_PATH, der);
    return { privateKey, publicKey, publicKeyDer: der };
  }
  // Load existing.
  const privatePem = readFileSync(PRIVATE_KEY_PATH, 'utf8');
  const privateKey = createPrivateKey({ key: privatePem, format: 'pem' });
  const publicKey = createPublicKey(privateKey);
  let publicKeyDer: Buffer;
  if (existsSync(PUBLIC_KEY_DER_PATH)) {
    publicKeyDer = readFileSync(PUBLIC_KEY_DER_PATH);
  } else {
    publicKeyDer = publicKey.export({ type: 'spki', format: 'der' }) as Buffer;
    writeFileSync(PUBLIC_KEY_DER_PATH, publicKeyDer);
  }
  return { privateKey, publicKey, publicKeyDer };
}
