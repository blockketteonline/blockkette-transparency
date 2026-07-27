import { useState, useEffect, useCallback } from 'react';
import {
  generateSeedPhrase,
  generateWalletsFromSeed,
  encrypt,
  decrypt,
  verifyPassword,
  createEncryptedBackup,
  isPasswordStrong,
} from '../utils/crypto';
import {
  saveWallet,
  loadWalletById,
  walletExists,
  getWalletIds,
  detectDeviceType,
  getCurrentWalletId,
  setCurrentWalletId,
  loadWalletMetadata,
  buildNetworksList,
} from '../utils/storage';
import { StoredWallet, Wallet, DeviceType, WalletAddresses } from '../types/wallet';

export interface UseWalletReturn {
  wallet: Wallet | null;
  loading: boolean;
  error: string | null;
  createWallet: (deviceType: DeviceType, password: string, isTestnet?: boolean) => Promise<Wallet>;
  unlockWallet: (walletId: string, password: string) => Promise<Wallet>;
  recoverWalletFromSeed: (
    seedPhrase: string,
    deviceType: DeviceType,
    password: string,
    isTestnet?: boolean
  ) => Promise<Wallet>;
  getWalletList: () => Promise<string[]>;
  getAddresses: () => WalletAddresses | null;
  logout: () => void;
  hasWallet: boolean;
  createBackup: (password: string) => Promise<string>;
  validatePassword: (password: string) => { valid: boolean; message: string };
  forceResyncChains: (password: string) => Promise<Wallet>;
}

const failedAttempts = new Map<string, number>();

// Schema version bumped any time a chain is added to generateWalletsFromSeed.
const CURRENT_SCHEMA_VERSION = '1.7.0';

// Every chain key that must have a *non-empty address* for a wallet to be
// considered "up to date". Checking `chain?.address` (not just `!!chain`)
// matters: a wallet can have a `chains.solayer` object sitting there with
// `address: ''` left over from a run where key generation partially failed
// (e.g. a broken crypto.ts import), and a presence-only check would treat
// that wallet as already-upgraded forever, silently leaving it broken.
const REQUIRED_CHAIN_KEYS = [
  'binance', 'tron', 'base', 'polygon', 'avalanche', 'arbitrum', 'optimism',
  'litecoin', 'xrp', 'ton', 'polkadot', 'near', 'fantom', 'dogecoin',
  'solayer', 'sonic', 'eclipse', 'solaxy', 'zx',
  'lumio', 'sonami', 'solieum', 'rome', 'termina',
  'hyperliquid', 'stellar', 'cardano', 'sui',
] as const;

function hasValidAddress(chains: any, key: string): boolean {
  return typeof chains?.[key]?.address === 'string' && chains[key].address.length > 0;
}

function needsChainUpgrade(storedVersion: string, chains: any): boolean {
  if (storedVersion !== CURRENT_SCHEMA_VERSION) return true;
  return REQUIRED_CHAIN_KEYS.some(key => !hasValidAddress(chains, key));
}

function mergeChains(existing: any, regenerated: any): Wallet['chains'] {
  const merged: any = {};
  const allKeys = new Set([...Object.keys(existing || {}), ...Object.keys(regenerated || {})]);
  for (const key of allKeys) {
    merged[key] = hasValidAddress(existing, key) ? existing[key] : regenerated[key];
  }
  return merged as Wallet['chains'];
}

export const useWallet = (): UseWalletReturn => {
  const [wallet, setWallet]       = useState<Wallet | null>(null);
  const [loading, setLoading]     = useState<boolean>(true);
  const [error, setError]         = useState<string | null>(null);
  const [deviceType, setDeviceType] = useState<DeviceType>('linux');
  const [hasWallet, setHasWallet] = useState<boolean>(false);

  useEffect(() => {
    const init = async () => {
      try {
        setDeviceType(detectDeviceType());
        const exists = await walletExists();
        setHasWallet(exists);
        const currentWalletId = getCurrentWalletId();
        if (currentWalletId) await loadWalletMetadata(currentWalletId);
      } catch (err) {
        console.error('Failed to check wallet existence:', err);
      } finally {
        setLoading(false);
      }
    };
    init();
  }, []);

  const validatePassword = useCallback((password: string): { valid: boolean; message: string } => {
    const strong = isPasswordStrong(password);
    if (strong) return { valid: true, message: '' };
    return {
      valid: false,
      message: 'Password must be at least 10 characters and contain an uppercase letter and a number.',
    };
  }, []);

  const createWallet = useCallback(
    async (selectedDevice: DeviceType, password: string, isTestnet = false): Promise<Wallet> => {
      setLoading(true);
      setError(null);
      const pwdCheck = validatePassword(password);
      if (!pwdCheck.valid) {
        setLoading(false);
        setError(pwdCheck.message);
        throw new Error(pwdCheck.message);
      }
      try {
        const network: 'mainnet' | 'testnet' = isTestnet ? 'testnet' : 'mainnet';
        const seedPhrase = generateSeedPhrase();

        let rawChains: any;
        try {
          rawChains = await generateWalletsFromSeed(seedPhrase, network);
        } catch (genErr) {
          // Surface this loudly — if key derivation itself is broken (e.g. a
          // bad crypto library import), silently producing a wallet with
          // missing/blank chain addresses is far worse than failing loudly.
          console.error('generateWalletsFromSeed failed during wallet creation:', genErr);
          throw new Error(
            `Wallet key generation failed: ${genErr instanceof Error ? genErr.message : String(genErr)}. ` +
            `No wallet was created — check the console for the underlying error.`
          );
        }
        const chains = rawChains as unknown as Wallet['chains'];

        // Sanity check: don't silently save a wallet with blank addresses.
        const missing = REQUIRED_CHAIN_KEYS.filter(k => !hasValidAddress(chains, k));
        if (missing.length > 0) {
          console.error('Wallet creation produced blank addresses for:', missing);
          throw new Error(
            `Wallet generation produced empty addresses for: ${missing.join(', ')}. ` +
            `This usually means a chain generator threw internally and was swallowed — check the console.`
          );
        }

        const newWallet: Wallet = {
          seedPhrase, chains,
          createdAt: Date.now(), deviceType: selectedDevice,
          bitcoinNetwork: network, networkMode: network,
        };

        const encryptedStr = await encrypt(
          JSON.stringify({ seedPhrase, chains, bitcoinNetwork: network, networkMode: network, version: CURRENT_SCHEMA_VERSION }),
          password
        );
        const stored: StoredWallet = {
          encryptedData: encryptedStr as unknown as StoredWallet['encryptedData'],
          deviceType: selectedDevice, createdAt: Date.now(), version: CURRENT_SCHEMA_VERSION,
          metadata: { walletCount: 1, networks: buildNetworksList(isTestnet) },
        };

        const id = await saveWallet(stored);
        setCurrentWalletId(id);
        setWallet(newWallet);
        setHasWallet(true);
        return newWallet;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to create wallet';
        setError(msg); throw new Error(msg);
      } finally { setLoading(false); }
    },
    [validatePassword]
  );

  const recoverWalletFromSeed = useCallback(
    async (seedPhrase: string, selectedDevice: DeviceType, password: string, isTestnet = false): Promise<Wallet> => {
      setLoading(true);
      setError(null);
      const pwdCheck = validatePassword(password);
      if (!pwdCheck.valid) {
        setLoading(false); setError(pwdCheck.message); throw new Error(pwdCheck.message);
      }
      const words = seedPhrase.trim().toLowerCase().split(/\s+/);
      if (words.length !== 12 && words.length !== 24) {
        setLoading(false);
        const msg = 'Seed phrase must be exactly 12 or 24 words.';
        setError(msg); throw new Error(msg);
      }
      try {
        const network: 'mainnet' | 'testnet' = isTestnet ? 'testnet' : 'mainnet';
        const cleanSeed = words.join(' ');

        let rawChains: any;
        try {
          rawChains = await generateWalletsFromSeed(cleanSeed, network);
        } catch (genErr) {
          console.error('generateWalletsFromSeed failed during recovery:', genErr);
          throw new Error(
            `Wallet key generation failed: ${genErr instanceof Error ? genErr.message : String(genErr)}. ` +
            `Check the console for the underlying error.`
          );
        }
        const chains = rawChains as unknown as Wallet['chains'];

        const missing = REQUIRED_CHAIN_KEYS.filter(k => !hasValidAddress(chains, k));
        if (missing.length > 0) {
          console.error('Wallet recovery produced blank addresses for:', missing);
          throw new Error(`Wallet recovery produced empty addresses for: ${missing.join(', ')}.`);
        }

        const recoveredWallet: Wallet = {
          seedPhrase: cleanSeed, chains,
          createdAt: Date.now(), deviceType: selectedDevice,
          bitcoinNetwork: network, networkMode: network,
        };

        const encryptedStr = await encrypt(
          JSON.stringify({ seedPhrase: cleanSeed, chains, bitcoinNetwork: network, networkMode: network, version: CURRENT_SCHEMA_VERSION }),
          password
        );
        const stored: StoredWallet = {
          encryptedData: encryptedStr as unknown as StoredWallet['encryptedData'],
          deviceType: selectedDevice, createdAt: Date.now(), version: CURRENT_SCHEMA_VERSION,
          metadata: { walletCount: 1, networks: buildNetworksList(isTestnet) },
        };

        const id = await saveWallet(stored);
        setCurrentWalletId(id);
        setWallet(recoveredWallet);
        setHasWallet(true);
        return recoveredWallet;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to recover wallet from seed phrase';
        setError(msg); throw new Error(msg);
      } finally { setLoading(false); }
    },
    [validatePassword]
  );

  const unlockWallet = useCallback(
    async (walletId: string, password: string): Promise<Wallet> => {
      setLoading(true);
      setError(null);
      const attempts = failedAttempts.get(walletId) || 0;
      if (attempts >= 5) throw new Error('Too many failed attempts. Please try again later.');
      try {
        const stored = await loadWalletById(walletId);
        if (!stored) throw new Error('Wallet not found');

        const encStr   = stored.encryptedData as unknown as string;
        const isValid  = await verifyPassword(encStr, password);
        if (!isValid) {
          failedAttempts.set(walletId, attempts + 1);
          throw new Error('Invalid password');
        }
        failedAttempts.delete(walletId);

        const walletData   = JSON.parse(await decrypt(encStr, password));
        const testnetFlag  = walletData.bitcoinNetwork === 'testnet' || walletData.networkMode === 'testnet';
        const network: 'mainnet' | 'testnet' = testnetFlag ? 'testnet' : 'mainnet';

        if (walletData.seedPhrase && walletData.chains) {
          const ex = walletData.chains;
          const storedVersion: string = walletData.version || '1.0.0';

          if (needsChainUpgrade(storedVersion, ex)) {
            try {
              const regenerated = await generateWalletsFromSeed(walletData.seedPhrase, network);
              walletData.chains = mergeChains(ex, regenerated);
              walletData.version = CURRENT_SCHEMA_VERSION;

              const stillMissing = REQUIRED_CHAIN_KEYS.filter(k => !hasValidAddress(walletData.chains, k));
              if (stillMissing.length > 0) {
                // Regeneration ran but some chains are STILL blank — this means
                // the generator itself is broken for those chains, not just
                // "this wallet predates them". Surface it instead of hiding it.
                console.error('Chain upgrade could not populate addresses for:', stillMissing);
              }

              const upgradedEncrypted = await encrypt(JSON.stringify(walletData), password);
              const { deleteWallet, saveWalletWithId } = await import('../utils/storage');
              await deleteWallet(walletId);
              await saveWalletWithId(walletId, {
                encryptedData: upgradedEncrypted as unknown as StoredWallet['encryptedData'],
                deviceType:    stored.deviceType,
                createdAt:     stored.createdAt,
                version:       CURRENT_SCHEMA_VERSION,
                metadata: {
                  walletCount: 1,
                  networks:    buildNetworksList(network === 'testnet'),
                },
              });
            } catch (upgradeErr) {
              // Previously this failure was swallowed with console.warn, which
              // meant a wallet could be stuck forever with blank addresses and
              // no visible signal why. Now we still let the user in with
              // whatever data we have, but the error is loud in the console.
              console.error('Wallet chain upgrade failed:', upgradeErr);
            }
          }
        }

        const loadedWallet: Wallet = {
          seedPhrase:     walletData.seedPhrase,
          chains:         walletData.chains as Wallet['chains'],
          createdAt:      stored.createdAt,
          deviceType:     stored.deviceType,
          bitcoinNetwork: walletData.bitcoinNetwork || 'mainnet',
          networkMode:    walletData.networkMode || walletData.bitcoinNetwork || 'mainnet',
        };

        setCurrentWalletId(walletId);
        setWallet(loadedWallet);
        return loadedWallet;
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Failed to unlock wallet';
        setError(msg); throw new Error(msg);
      } finally { setLoading(false); }
    },
    []
  );

  // Manual escape hatch: force a full chain regeneration + merge right now,
  // regardless of the stored version flag. Useful if a wallet's version was
  // marked upgraded in a prior session even though some addresses ended up
  // blank (e.g. from a save that partially failed). Wire this to a "Fix
  // missing addresses" button in Settings if you want a user-facing repair
  // action instead of clearing the wallet and starting over.
  const forceResyncChains = useCallback(
    async (password: string): Promise<Wallet> => {
      if (!wallet) throw new Error('No wallet loaded');
      const network: 'mainnet' | 'testnet' = wallet.networkMode === 'testnet' || wallet.bitcoinNetwork === 'testnet' ? 'testnet' : 'mainnet';
      const regenerated = await generateWalletsFromSeed(wallet.seedPhrase, network);
      const merged = mergeChains(wallet.chains, regenerated);

      const missing = REQUIRED_CHAIN_KEYS.filter(k => !hasValidAddress(merged, k));
      if (missing.length > 0) {
        throw new Error(`Resync could not produce addresses for: ${missing.join(', ')}. Check the console.`);
      }

      const updatedWallet: Wallet = { ...wallet, chains: merged };
      const walletId = getCurrentWalletId();
      if (walletId) {
        const stored = await loadWalletById(walletId);
        if (stored) {
          const encryptedStr = await encrypt(
            JSON.stringify({
              seedPhrase: updatedWallet.seedPhrase,
              chains: updatedWallet.chains,
              bitcoinNetwork: updatedWallet.bitcoinNetwork,
              networkMode: updatedWallet.networkMode,
              version: CURRENT_SCHEMA_VERSION,
            }),
            password
          );
          const { saveWalletWithId } = await import('../utils/storage');
          await saveWalletWithId(walletId, {
            encryptedData: encryptedStr as unknown as StoredWallet['encryptedData'],
            deviceType: stored.deviceType,
            createdAt: stored.createdAt,
            version: CURRENT_SCHEMA_VERSION,
            metadata: { walletCount: 1, networks: buildNetworksList(network === 'testnet') },
          });
        }
      }
      setWallet(updatedWallet);
      return updatedWallet;
    },
    [wallet]
  );

  const getAddresses = useCallback((): WalletAddresses | null => {
    if (!wallet) return null;
    const c = wallet.chains;
    return {
      ethereum:  c.ethereum?.address ?? '',
      bitcoin:   c.bitcoin?.address ?? '',
      solana:    c.solana?.address ?? '',
      binance:   c.binance?.address ?? '',
      tron:      c.tron?.address ?? '',
      base:      c.base?.address ?? '',
      polygon:   c.polygon?.address ?? '',
      avalanche: c.avalanche?.address ?? '',
      arbitrum:  c.arbitrum?.address ?? '',
      optimism:  c.optimism?.address ?? '',
      litecoin:  c.litecoin?.address ?? '',
      xrp:       c.xrp?.address ?? '',
      ton:       c.ton?.address ?? '',
      polkadot:  c.polkadot?.address ?? '',
      near:      c.near?.address ?? '',
      fantom:    c.fantom?.address ?? '',
      dogecoin:  c.dogecoin?.address ?? '',
      solayer:   c.solayer?.address ?? '',
      sonic:     c.sonic?.address ?? '',
      eclipse:   c.eclipse?.address ?? '',
      solaxy:    c.solaxy?.address ?? '',
      zx:        c.zx?.address ?? '',
      lumio:     c.lumio?.address ?? '',
      sonami:    c.sonami?.address ?? '',
      solieum:   c.solieum?.address ?? '',
      rome:      c.rome?.address ?? '',
      termina:   c.termina?.address ?? '',
      hyperliquid: c.hyperliquid?.address ?? '',
      stellar:     c.stellar?.address ?? '',
      cardano:     c.cardano?.address ?? '',
      sui:         c.sui?.address ?? '',
      bitcoinNetwork: wallet.bitcoinNetwork,
    };
  }, [wallet]);

  const getWalletList  = useCallback(async (): Promise<string[]> => getWalletIds(), []);

  const createBackup   = useCallback(async (password: string): Promise<string> => {
    if (!wallet) throw new Error('No wallet loaded');
    return JSON.stringify(await createEncryptedBackup(wallet, password));
  }, [wallet]);

  const logout = useCallback((): void => {
    setWallet(null);
    setCurrentWalletId(null);
  }, []);

  return {
    wallet, loading, error,
    createWallet, unlockWallet, recoverWalletFromSeed,
    getWalletList, getAddresses, logout,
    hasWallet, createBackup, validatePassword,
    forceResyncChains,
  };
};