# Genesis Bonding Curve – Smart Contract

**The mathematical engine that provides unlimited on-chain liquidity for trading, perps, and AI‑powered strategies.**  
Part of the Blockkette protocol – a fully non‑custodial super app.

---

## 📐 Mathematical Model

The Genesis Bonding Curve is a **linear price function** that deterministically links the token supply to its price:

\[
P(S) = \text{base\_price} + \text{price\_increment} \times S
\]

- **`base_price`** – the initial token price when supply \( S = 0 \).
- **`price_increment`** – the constant increase in price for each additional token minted (or decrease when burned).

### Buy (mint) calculation

The total SOL cost to mint \( x \) tokens when the current supply is \( S \) is the integral of the price function:

\[
\text{cost}(S, x) = \int_{S}^{S+x} \! (\text{base\_price} + \text{price\_increment} \cdot t) \, dt
\]

This evaluates to:

\[
\text{cost}(S, x) = \text{base\_price} \cdot x + \frac{\text{price\_increment}}{2} \left[ (S + x)^2 - S^2 \right]
\]

The contract computes this **exactly** in `bonding_curve_buy()` (see `math.rs`), using 128‑bit arithmetic to avoid overflow.

### Sell (burn) calculation

Selling \( x \) tokens returns SOL based on the area under the price curve from \( S-x \) to \( S \):

\[
\text{return}(S, x) = \text{base\_price} \cdot x + \frac{\text{price\_increment}}{2} \left[ S^2 - (S - x)^2 \right]
\]

This guarantees a **deterministic, always‑available bid** – there is no order book and no liquidity provider to run away.

### Why “unlimited liquidity”?

Because the bonding curve **mints tokens when you buy and burns them when you sell**, the supply \( S \) can change arbitrarily. The curve itself **always exists** – it’s just a mathematical function. The only constraint is the `max_supply`, which can be adjusted by governance. There is no AMM pool that can be drained; the price simply adjusts with supply.  
This means **anyone can always buy or sell**, and the price reflects real demand.

---

## ⚡ Powering Perpetuals (Perps)

The bonding curve’s deterministic pricing makes it ideal as a **collateral engine** for perpetual futures:

- The SOL vault acts as a **guaranteed counterparty** – trades are settled directly against the curve.
- Because the price is a pure function of supply, funding rates can be computed **on‑chain without oracles**.
- Liquidations are instant – the contract can always buy/sell at the curve price.
- The Perps module (see `PerpsPage.tsx` in the frontend) will use this bonding curve as the **base liquidity layer**, allowing users to open leveraged positions directly against the curve’s SOL vault.

---

## 🤖 AI Integration – Analyze Before You Execute

The **Blockkette AI Advisor** (`ai_advisor.py` + `AIAdvisorPanel.tsx`) provides real‑time market analysis with hard 2:1 reward:risk rules.  

The user flow:
1. **AI analyzes** 4H price data, strategies, and indicators.
2. **AI returns** a long/short decision with exact entry, stop, and take‑profit.
3. **User reviews** the analysis in the wallet.
4. **User executes** the trade on the Perps module, which is backed by this bonding curve.

Because the AI is trained on real market data (and will be upgraded to custom ML models), users get **professional‑grade analysis** before touching the curve.

---

## 💎 GNS Token – Subscription Access

Access to the AI and advanced Perps features is **token‑gated by the Genesis (GNS) token**:

- Staking GNS unlocks higher AI call limits, real‑time signals, and reduced fees.
- GNS itself is the **token of this bonding curve** – its price is set by the formulas above.
- Revenue from subscription staking flows back into the protocol, creating a **self‑sustaining ecosystem**.

---

## 🏛 Upcoming DEX – 100 Blockchains, One Bonding Curve

The Genesis DEX will extend this single bonding curve across **100+ blockchains** using cryptographic bridges:

- Cross‑chain swaps are routed through the SOL vault and equivalent vaults on other chains.
- The curve ensures **infinite liquidity** because it never runs out – it only changes price.
- Auditable, on‑chain proof that every trade follows the mathematical rules.

The DEX will be the **single liquidity hub for the entire multichain ecosystem**, removing fragmentation entirely.

---

## 🔒 Security & Verifiability

- All bonding curve math is **on‑chain and deterministic** – anyone can compute the expected price from the current supply.
- Admin changes (curve parameters) are **timelock‑protected**.
- The contract is auditable (see the source). A professional institutional audit is planned post‑YC.

---

## 📂 Where to look

- `contracts/genesis-bonding-curve/src/instructions/bonding_curve.rs` – The core buy/sell logic shown here.
- `contracts/genesis-bonding-curve/src/math.rs` – The exact arithmetic functions.
- `contracts/genesis-bonding-curve/src/state/bonding_curve.rs` – The account structure.

*(The full Anchor project is in `genesis_token/` on the server; this repository holds the key instruction file.)*

---

*Built by the Blockkette team. Testnet only.*
