# Blockkette – Non‑Custodial Wallet Core

**This repository contains the wallet’s core hook (`useWallet.tsx`), which handles all key generation, encryption, and storage entirely in the browser.  
The backend and AI modules never see private keys, passwords, or any sensitive data – the wallet is fully self‑sovereign.**

---

## 🔐 Non‑Custodial Security Model

### 1. Keys never leave the browser
- Wallet creation (`createWallet`) uses **`generateWalletsFromSeed`** to derive keys for 31+ blockchains **directly in the user’s browser** (client‑side JavaScript).
- The seed phrase is **never transmitted** to any server, API, or external service.

### 2. Local encryption with user‑chosen password
- The seed phrase and all derived private keys are **encrypted with AES‑GCM** using a password chosen by the user (see `encrypt` in `crypto.tsx`).
- The encrypted blob is stored in **IndexedDB** (the browser’s local storage), never sent over the network.

### 3. Password is never shared
- The user’s password is used to **derive an encryption key** (PBKDF2). The password itself is never stored – only the encrypted data.
- To unlock the wallet, the user provides the password **locally**, and the app decrypts the data in‑memory. The password is **never sent to any backend**.

### 4. Zero‑knowledge architecture
- The backend APIs (swap, AI, prices) receive **only public information** (e.g., market, margin, transaction details).  
  They have **no access to private keys, seed phrases, or even wallet addresses** beyond what is required to broadcast a transaction.
- Transaction signing happens **exclusively in the browser** using the decrypted private key – the backend cannot sign anything on behalf of the user.

### 5. Auditable from the browser
Anyone can verify the wallet’s security by opening the browser’s Developer Tools (F12) and:

- Inspecting network requests while creating or unlocking a wallet – **no network request contains the password or seed**.
- Viewing the **IndexedDB** storage (Application > IndexedDB) and confirming the encrypted blob – it’s unintelligible without the password.
- Checking the source code in this repository to confirm the encryption and key‑derivation logic.

---

## 🧩 What’s in this repo?

| File | Purpose |
|------|---------|
| `frontend/src/hooks/useWallet.tsx` | Core wallet hook – create, unlock, recover, backup, chain upgrade |
| `frontend/src/utils/crypto.tsx` | Encryption/decryption, key derivation, password validation |
| `frontend/src/utils/storage.tsx` | IndexedDB persistence layer |

*(Other modules like the AI advisor, perps, and smart contracts are in separate directories.)*

---

## 🏛 Cryptographic Details

- **Symmetric encryption:** AES‑256‑GCM  
- **Key derivation:** PBKDF2 with 100,000 iterations  
- **Password strength:** Enforced minimum length, uppercase, number  
- **Seed phrase:** BIP‑39 (12‑word) used as the master seed for all chains  

---

## 🔒 Why Blockkette cannot access user funds

1. **We don’t have the seed phrase** – it’s generated locally and encrypted with a password we never receive.
2. **We don’t have the password** – the password is never sent to our servers; it’s used only client‑side.
3. **We can’t sign transactions** – private keys are decrypted in the browser for signing and immediately discarded from memory after use.
4. **Our backend is stateless** – it stores no wallet data, no encrypted blobs, no user identifiers.

Every transaction is signed **exclusively by the user’s own device**. Even if the Blockkette backend were compromised, there would be nothing to steal – no keys, no passwords, no funds.

---

## 📡 Live Demo

The wallet and AI advisor are live on **testnet**:  
`https://blockkette.online`

You can open the site, create a wallet (completely in‑browser), and inspect network traffic to confirm that no private data leaves your machine.

---

*Built by the Blockkette team. Not financial advice. Testnet only.*
