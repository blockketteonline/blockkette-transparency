// ciphervault-frontend/src/lib/idb.ts
// ── Minimal IndexedDB key-value store ──────────────────────────────────────
// Drop-in-shaped replacement for the localStorage.getItem/setItem pattern
// used elsewhere in the app (see cv_activity / cv_perps_locked_native in
// WalletDashboardPage.tsx), for pages where wallet-connected data should live
// in IndexedDB instead. IndexedDB is asynchronous and has a much higher
// storage ceiling than localStorage, which matters once you're persisting
// per-wallet trade history, cached on-chain state, etc.
//
// Usage:
//   await idbSet('genesis', 'activity', activityArray)
//   const activity = await idbGet<ActivityItem[]>('genesis', 'activity')
//
// Each "store" is a logical namespace (e.g. one per page/feature) so
// GenesisPage and BusinessFactoryPage don't collide with each other or with
// anything else that adopts this later.

const DB_NAME = 'blockette_wallet';
const DB_VERSION = 1;

// All logical stores must be declared up front — IndexedDB only creates
// object stores during a version-change (onupgradeneeded), not on demand.
const STORE_NAMES = ['genesis', 'business_factory'] as const;
type StoreName = (typeof STORE_NAMES)[number];

let dbPromise: Promise<IDBDatabase> | null = null;

function openDb(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      for (const name of STORE_NAMES) {
        if (!db.objectStoreNames.contains(name)) {
          db.createObjectStore(name);
        }
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error ?? new Error('Failed to open IndexedDB'));
  });
  return dbPromise;
}

export async function idbGet<T>(store: StoreName, key: string): Promise<T | null> {
  try {
    const db = await openDb();
    return await new Promise<T | null>((resolve, reject) => {
      const tx = db.transaction(store, 'readonly');
      const req = tx.objectStore(store).get(key);
      req.onsuccess = () => resolve((req.result as T | undefined) ?? null);
      req.onerror = () => reject(req.error ?? new Error('IndexedDB read failed'));
    });
  } catch {
    return null;
  }
}

export async function idbSet<T>(store: StoreName, key: string, value: T): Promise<void> {
  try {
    const db = await openDb();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(store, 'readwrite');
      tx.objectStore(store).put(value, key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error ?? new Error('IndexedDB write failed'));
    });
  } catch {
    // Storage failures shouldn't crash the UI — callers treat this as best-effort persistence.
  }
}

export async function idbDelete(store: StoreName, key: string): Promise<void> {
  try {
    const db = await openDb();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(store, 'readwrite');
      tx.objectStore(store).delete(key);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error ?? new Error('IndexedDB delete failed'));
    });
  } catch {
    // best-effort, same as idbSet
  }
}
