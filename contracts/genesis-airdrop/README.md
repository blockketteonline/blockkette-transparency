# Genesis Airdrop – Community Growth Engine

**A Merkle‑tree‑based token distribution that unlocks liquidity only when the Genesis Bonding Curve reaches a $1 billion market cap – aligning community incentives with real protocol value.**

---

## 🪂 Why an Airdrop?

Airdrops are the most powerful tool in Web3 to **bootstrap a global community**.  
But most airdrops fail because they distribute tokens **before** the protocol has real utility, leading to immediate sell‑pressure and a race to the bottom.

The Genesis Airdrop is different:  
- Tokens are **allocated at Genesis** but **cannot be claimed** until the protocol’s market cap exceeds a predefined threshold (e.g., $1 billion).
- This ensures the airdrop **rewards long‑term believers** and only becomes liquid when the ecosystem is already thriving.

---

## 🔢 Mathematical Mechanism

### Merkle Tree Allocation
Each eligible user is assigned a specific amount of GNS tokens.  
A Merkle root is stored on‑chain – users claim by providing a Merkle proof that they are entitled to the amount.

### Market Cap Gate
The claim function enforces a **real‑time market cap check** using the Bonding Curve and the on‑chain SOL/USD oracle:

\[
\text{MarketCap}_{\text{USD}} = \frac{\text{TotalSupply} \times \text{CurrentPrice}_{\text{SOL}} \times \text{SOL/USD}_{\text{1e6}}}{10^6}
\]

Where:
- `TotalSupply` = current GNS supply (from the Bonding Curve state)
- `CurrentPrice_SOL` = current GNS price in SOL (computed from the bonding curve formula)
- `SOL/USD_1e6` = on‑chain SOL price with 6‑decimal precision

If the calculated market cap is **below the required cap** (`required_market_cap_usd_1e6`), the claim is **reverted**.

### Example
If the required market cap is $1,000,000,000 ($1B), and the current market cap is $500M, claims are blocked.  
Once organic demand drives the bonding curve to a total market cap of $1B, all remaining airdrop allocations become immediately claimable.

---

## 💧 The Liquidity Illusion – Why This Works

Crucially, the airdrop tokens are **already minted** into the escrow at initialization – they count toward `total_supply`.  
This means the bonding curve **already prices them in** from day one.

However, because they are **locked in escrow and not circulating**, there is no sell‑side pressure until the market cap target is hit.  
The result:
- **Early buyers are not diluted** – the airdrop supply is already factored into the price.
- **The market cap target acts as a credibility signal** – it proves the protocol has real usage before rewards are unlocked.
- **Community members are incentivized to grow the protocol** – their allocation becomes valuable only when the ecosystem succeeds.

---

## 🌱 Community Growth Strategy

The Genesis Airdrop will be used to:

1. **Reward early adopters** – users who create wallets, test Perps, and provide feedback.
2. **Incentivize developers** – builders who integrate with the Genesis DEX or deploy business tokens.
3. **Onboard institutions** – banks and payment providers that use the Business Token Factory.
4. **Grow social presence** – engagement on X, Discord, Telegram.

A **multi‑phase airdrop** with increasing market cap thresholds can be deployed, ensuring continuous community alignment as the protocol scales.

---

## 🔗 How It Ties Into the Bonding Curve

- The airdrop escrow is **funded from the same GNS mint** that backs the bonding curve.
- Every claim **reduces escrow balance** but does not mint new tokens – the total supply is unchanged, preserving the curve’s integrity.
- The market cap gate ensures that the airdrop **only releases value into a liquid, high‑demand environment**, preventing price crashes.
- The bonding curve’s mathematical certainty guarantees that **all GNS (including airdrop allocations) will always be tradeable** – there is no cliff or lockup that could break the curve.

---

## 🏛 Why This Economic Model Is Realistic

- **Backed by real utility:** The Genesis ecosystem already has a live wallet, AI advisor, Perps terminal, and business token factory – all demanding GNS for fees, subscriptions, and token creation.
- **Infinite liquidity:** The bonding curve ensures anyone can always buy or sell GNS at a fair, deterministic price – no exchange listing required.
- **Community‑owned:** The airdrop distributes ownership to the users who actually build the network, aligning incentives for the long term.

A $1B market cap is ambitious but **achievable** given the scope: a non‑custodial super app with 31+ chains, a DEX with unlimited liquidity, an AI trading engine, and a business token factory – all powered by GNS.

---

## 📂 Where to look

- `contracts/genesis-airdrop/src/instructions/airdrop.rs` – The airdrop initialization and claim logic.
- `contracts/genesis-bonding-curve/` – The underlying bonding curve (market cap calculation).
- `contracts/genesis-bonding-curve/src/math.rs` – `compute_market_cap_usd_1e6`.

*(The full Anchor project is in the main `genesis_token/` repository.)*

---

*Built by the Blockkette team. Not financial advice. Testnet only.*
