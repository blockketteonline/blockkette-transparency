# Genesis USDT Swap – The Bridge to a 100‑Chain DEX

**The first on‑chain swap pair (GNS → USDT) that uses the Genesis Bonding Curve for infinite liquidity – and the blueprint for a DEX connecting 100+ blockchains.**

---

## 💱 What the USDT Swap Does

The `swap_gns_for_usdt` instruction allows users to:

1. Burn GNS tokens
2. Withdraw the equivalent SOL from the bonding curve's SOL vault
3. Convert that SOL to USDT using the on‑chain SOL/USD oracle
4. Receive USDT from the protocol's USDT vault

This is the first **non‑SOL exit** from the Genesis ecosystem – a crucial step toward becoming a universal DEX.

---

## 🔮 The Vision: A 100‑Chain DEX with Unlimited Liquidity

Our research goal is to build a **fully decentralised cross‑chain DEX** where any asset on any chain can be swapped for any other asset, all settled through the **Genesis Bonding Curve**.

The bonding curve provides **infinite liquidity** because it mints/burns GNS on every trade – there is no pool that can be drained.  
By replicating this logic across multiple chains and connecting them with a trust‑minimised bridge, we can create a **single liquidity hub for the entire blockchain industry**.

---

## 🧠 Realistic Computer Science: How 100 Chains Talk to Each Other

Connecting 100 blockchains is a hard problem, but solvable with existing cryptographic primitives. Our planned architecture:

### 1. Canonical GNS on Every Chain
Using a **cross‑chain message passing protocol** (e.g., Wormhole, Axelar, or a custom light‑client bridge), we deploy a canonical GNS token on each supported chain. The supply across chains is pegged by a network of **bridge validators** or **threshold signatures** that lock GNS on Solana and mint wrapped GNS on the target chain.

### 2. Local Bonding Curve Mirrors
On each target chain, we deploy a **mirror bonding curve** that uses the local GNS as its base token. When a user swaps, say, Ethereum USDC for Polygon MATIC, the trade is broken into two legs:
- ETH USDC → GNS (on Ethereum, via the local curve)
- GNS → MATIC (on Polygon, via that local curve)

The bridging of GNS between chains happens atomically via the message protocol.

### 3. Unified Pricing via Oracle Network
A **decentralised oracle network** (like Pyth or Switchboard) feeds the same GNS/USD price to all curves, ensuring consistent pricing regardless of chain.

### 4. Trust‑Minimised Bridging
Instead of relying on a single bridge, we can use **light client verification** (e.g., Solana light client on Ethereum, Ethereum light client on Solana) so that each chain can independently verify state transitions on the other chain – no third‑party trust required.

---

## ♾️ Why This Creates Unlimited Liquidity

- The bonding curve is **mathematically inexhaustible** – it always buys and sells at a deterministic price.
- Because GNS is the **intermediate asset** for all cross‑chain trades, the demand for GNS scales with total DEX volume across all chains.
- Arbitrageurs keep the price aligned across chains, maintaining a single global GNS price.

The result: a DEX that can handle billions in daily volume without liquidity providers, without order books, and without front‑running.

---

## 💎 How This Makes GNS Highly Valuable

- **Universal Fee Token:** Every cross‑chain swap burns GNS or pays fees in GNS.
- **Staking Rewards:** A portion of bridge fees flow to the staking pool, rewarding long‑term holders.
- **Scarcity:** As more chains are added, more GNS is locked in bridge contracts and bonding curves across chains, reducing circulating supply.
- **Network Effect:** The more chains connected, the more indispensable GNS becomes as the liquidity backbone – a self‑reinforcing cycle.

---

## 📐 Research & Implementation Plan

- **Phase 1 (Current):** On‑chain USDT swap via SOL vault + oracle.
- **Phase 2 (Post‑YC):** Deploy a mirror bonding curve on one EVM chain (e.g., Ethereum Sepolia) with a proof‑of‑concept bridge.
- **Phase 3:** Integrate a production bridge (Wormhole or Axelar) and add 5–10 major chains.
- **Phase 4:** Implement light‑client verification for trust‑minimised bridging; scale to 100+ chains.

---

## 📂 Where to look

- `contracts/genesis-usdt-swap/src/instructions/usdt_swap.rs` – The GNS→USDT swap logic.
- `contracts/genesis-bonding-curve/` – The infinite liquidity engine.
- `contracts/genesis-oracle/` – The on‑chain price feeds.

*Built by the Blockkette team. Testnet only.*
