# Genesis On‑Chain Oracle

**Stores and updates the SOL/USD and GNS/USD prices on‑chain, providing the exchange rates that power the entire Genesis economy.**

---

## 🔍 What Are These Oracles?

| Oracle | Price Stored | Used By |
|--------|-------------|---------|
| `SolUsdOracle` | SOL price in USD (6‑decimal precision) | Bonding curve market cap, airdrop gate |
| `GnsUsdOracle` | GNS price in USD (6‑decimal precision) | Factory config (USD → GNS conversion), subscription pricing |

Both oracles live entirely **on‑chain** as Solana accounts, updated by an **authorized oracle authority**.

---

## 🔢 Why They Matter

### Market Cap Calculation (Airdrop Gate)
The bonding curve’s market cap in USD is calculated as:

\[
\text{MarketCap}_{\text{USD}} = \frac{\text{TotalSupply} \times \text{GNSPrice}_{\text{SOL}} \times \text{SOLPrice}_{\text{USD}}}{10^{12}}
\]

The airdrop uses this to check whether the required market cap (e.g., $1B) has been reached before allowing claims – ensuring the token is truly liquid before rewards are released.

### Fee Conversion (Business Factory & Subscriptions)
The **Factory Config** stores `gns_usd_price_cents`, which is updated from the `GnsUsdOracle`. When a business creates a token and pays a fee in USD, the contract converts it to GNS:

\[
\text{GNS cost} = \frac{\text{USD fee} \times 100}{\text{gns\_usd\_price\_cents}}
\]

This guarantees that fees remain constant in USD terms, regardless of GNS volatility.

### AI Subscription Pricing
The subscription module reads `ai_subscription_cost_gns` from the Factory Config, which is itself derived from the GNS/USD oracle – again ensuring stable USD pricing.

---

## 🛡 Security & Staleness

- The **update authority** is the only account allowed to push new prices, and it can be a **multi‑sig or an automated keeper bot**.
- Every consumer of the oracle (airdrop, config) checks a **staleness threshold** (`ORACLE_MAX_STALENESS_SECS`). If the price hasn’t been updated recently, operations revert – preventing the use of outdated data.
- The oracle authority can be updated by the protocol admin, allowing a transition to a fully decentralized oracle network (e.g., Pyth or Switchboard) in the future.

---

## 📂 Where to look

- `contracts/genesis-oracle/src/instructions/oracle.rs` – The oracle initialization and update logic.
- `state/oracle.rs` – The account structures (`SolUsdOracle`, `GnsUsdOracle`).
- `constants.rs` – The staleness threshold.

*Built by the Blockkette team. Testnet only.*
