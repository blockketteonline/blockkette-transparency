# IndexedDB Storage Module (`idb.ts`)

This module provides **key‑value storage backed by IndexedDB** – the browser’s built‑in, asynchronous database. It is used by the wallet to persist **encrypted wallet data**, trading history, and other per‑wallet state.

---

## 🔐 Why IndexedDB instead of localStorage?

| Feature | localStorage | IndexedDB |
|--------|--------------|-----------|
| Storage limit | ~5–10 MB | Typically 50% of disk space (hundreds of MB) |
| Data types | Strings only | Any structured cloneable data (objects, arrays, binary) |
| Async | No (synchronous) | Yes (non‑blocking) |
| Quota pressure | Can be evicted by browser | More persistent |

The wallet stores **encrypted seed phrases and private keys** – data that can grow significantly as the user adds accounts across 31+ chains. IndexedDB’s larger quota and structured storage make it the correct choice.

---

## 🔒 How the data is protected

- **All data stored is already encrypted** with AES‑256‑GCM **before** it reaches IndexedDB.  
  The key is derived from the user’s password via PBKDF2, which is never stored.
- The database name (`blockette_wallet`) and object stores (`genesis`, `business_factory`) contain **only opaque, encrypted blobs** – they are unintelligible without the password.
- Even if an attacker gains access to the raw IndexedDB files, they cannot decrypt the data without the password (which is never saved on disk or sent over the network).

---

## 🧠 How to verify from the browser

1. Open the Blockkette wallet at `https://blockkette.online`.
2. Press `F12` to open Developer Tools.
3. Go to the **Application** tab.
4. In the left sidebar, expand **IndexedDB** → **blockette_wallet**.
5. You will see one or more object stores (e.g., `genesis`).  
   Inside each store, the values are **encrypted strings** – they look like random characters, not like readable keys or addresses.
6. Compare with the source code in this repository – the `idbSet` function writes the encrypted wallet blob (created in `useWallet.tsx`) directly to IndexedDB.

There is **no localStorage** used for sensitive data – only IndexedDB.

---

## 📂 Files

- `idb.ts` – The module itself (exports `idbGet`, `idbSet`, `idbDelete`).

---

*The encryption and key‑derivation logic is in `frontend/src/utils/crypto.tsx`. The wallet hook that orchestrates everything is `frontend/src/hooks/useWallet.tsx`.*
