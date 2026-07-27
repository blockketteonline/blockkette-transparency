# Genesis Business Token Factory

**Empowering businesses, banks, and communities to launch their own branded tokens – powered by the Genesis Bonding Curve and GNS token.**

---

## 🏭 What is the Business Token Factory?

The Business Token Factory allows anyone to create a **customised, fully on‑chain token** with a dedicated bonding curve.  
The token is immediately tradeable, and its price is determined by the same mathematical model that drives the Genesis DEX – guaranteeing **always‑available liquidity**.

Businesses pay in **GNS tokens** to mint their token. This creates **constant demand for GNS**, which flows directly into the core Genesis Bonding Curve, strengthening the entire ecosystem.

---

## 🔢 How It Works (Mathematical Model)

### 1. Pricing Tiers
The factory has multiple **tiers**, each with a USD cost and a maximum token supply.  
To create a token, the user picks a tier. The USD cost is converted to GNS using the protocol’s on‑chain price feed:

\[
\text{gns\_cost} = \frac{\text{tier\_usd\_cost} \times 100}{\text{gns\_usd\_price\_cents}}
\]

This ensures the fee is **always exactly the USD value**, regardless of GNS volatility.

### 2. Token’s Own Bonding Curve
Once created, the business token runs on a **dedicated bonding curve** with its own `base_price`, `price_increment`, and `max_supply`.  
The price formula is identical to the main Genesis curve:

\[
P(S) = \text{base\_price} + \text{price\_increment} \times S
\]

- **`base_price`** – the starting price when supply \( S = 0 \).
- **`price_increment`** – the slope; every additional token minted increases the price by this amount.

Because the curve is mathematical, every buy or sell is **instantly executable** – no need for market makers.  
The token’s SOL vault guarantees liquidity at all times.

### 3. Upgrading & Customisation
Businesses can **upgrade** their tier by paying the difference in GNS (again converted at the current GNS/USD price).  
They can also adjust their curve parameters (`base_price`, `price_increment`, `max_supply`) later, allowing the token to evolve with the business.

### 4. Fee Structure
Every token creation, upgrade, and mint transaction **pays GNS** into the protocol’s fee vault.  
These fees are then used to **buy and burn GNS from the main bonding curve**, creating deflationary pressure and rewarding all GNS holders.

---

## 🏦 How It Helps Businesses

### 💳 Instant Customer Loyalty Tokens
A coffee shop can issue its own token and give customers **digital loyalty points** that are automatically liquid on the bonding curve.  
Customers can trade them for SOL, hold them, or use them for discounts – creating a **closed‑loop economy** around the business.

### 📈 Faster Sales & Community Bonding
- Businesses can run **token‑gated promotions** (e.g., “hold 100 CoffeeTokens for 10% off”).
- Tokens become a **vehicle for customer ownership** – as the business grows, the token price increases on the curve.
- This aligns incentives: customers who hold tokens benefit from the business’s success, turning them into **brand ambassadors**.

### 🌍 Economic Empowerment
For emerging economies or local communities, the factory provides a way to **launch a local currency** backed by real SOL liquidity.  
This can:
- Increase **purchasing power** by giving communities a stable, tradeable asset.
- Allow **micro‑businesses** to raise capital without a bank.
- Create a **transparent, auditable** financial system on‑chain.

---

## 🏦 The Vision: A Blockchain‑Based Bank

We plan to integrate **payment services** directly into this smart contract:

- Businesses can accept their own token as payment via the Blockkette wallet.
- The wallet’s **swap module** will instantly convert any token to SOL/USDC using the bonding curve.
- This eliminates payment processors, reduces fees, and gives merchants **near‑instant settlement**.

Further, we are designing a **“Genesis Bank” module** that will allow businesses to:
- Accept multi‑currency payments (their token + SOL + stablecoins).
- Automatically manage liquidity and price discovery via their bonding curve.
- Provide lending services (users stake their tokens to borrow SOL, all on‑chain).

This turns every business token into a **micro‑economy** – a self‑contained financial ecosystem that can scale globally.

---

## 🔗 How It Strengthens the Genesis Ecosystem

1. **Every token created burns GNS** – increasing scarcity.
2. **Every business token uses the same bonding curve math** – auditable and predictable.
3. **A network effect:** the more businesses join, the more demand for GNS, pushing up its price on the main curve.
4. **Cross‑tradeability:** eventually, the DEX will allow swapping between any business token and any other asset on 100+ blockchains – all settled through the core Genesis curve.

This is the **foundation of a decentralised, self‑sustaining global economy** – where every brand, every creator, and every community can have their own sovereign money, yet all are interconnected through a single, verifiable liquidity engine.

---

## 📂 Where to look

- `contracts/genesis-business-factory/src/instructions/business_token.rs` – The core factory logic (create, upgrade, update parameters).
- `contracts/genesis-bonding-curve/` – The underlying bonding curve (used by every business token).
- `contracts/genesis-bonding-curve/src/math.rs` – The arithmetic functions (`bonding_curve_buy`, `bonding_curve_sell`, `usd_to_gns`).

*(The full Anchor project is in the main `genesis_token/` repository.)*

---

*Built by the Blockkette team. Testnet only. The future of economics is programmable.*
