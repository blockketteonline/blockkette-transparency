# Genesis Point‑of‑Sale (POS) – Decentralised Payment Engine

**The on‑chain payment system that lets customers pay in SOL, receive business tokens, and rewards merchants with GNS – all fully non‑custodial and integrated with the Genesis Bonding Curve.**

---

## 🛒 How It Works

### 1. Merchant Activates POS
A business that already has a **Business Token** (created via the Business Token Factory) activates the POS module by paying a one‑time GNS fee. This creates a `MerchantPOS` account linked to the business’s token mint.

### 2. Customer Pays
A customer sends a **SOL payment** directly to the smart contract. The transaction:
- Splits the SOL into two parts: the **merchant’s share** and a **protocol fee**.
- The merchant’s share **buys business tokens** from the business’s own bonding curve – the customer receives those tokens instantly.
- The protocol fee **buys GNS** from the main Genesis Bonding Curve – half of that GNS is sent to the merchant as a **fee refund**, half goes to the fee vault.

### 3. Settlement is Instant
No payment processor, no bank delay. The blockchain confirms the payment in seconds, and both the customer (with their tokens) and the merchant (with GNS rewards) are immediately credited.

---

## 🔗 How the Blockkette Wallet Connects

The Blockkette non‑custodial wallet is the **native interface** for this POS system:

- **Merchants** manage their business token, view transaction history, and track GNS rewards directly in the wallet.
- **Customers** can pay with a single tap – the wallet generates and signs the `process_pos_payment` transaction.
- **Both parties** see real‑time balances of business tokens, GNS, and SOL, all in one place.

Because the wallet is non‑custodial, neither Blockkette nor any intermediary ever holds the funds. The smart contract enforces every transfer.

---

## 🏦 Banking & Economic Scaling

This POS system is the first step toward a **decentralised global bank** running on the Genesis protocol.

### For Businesses
- Accept payments **anywhere in the world** without a merchant account.
- Issue **branded loyalty tokens** that automatically increase in value via their bonding curve.
- Receive **GNS fee rebates** with every sale – building a stake in the Genesis ecosystem.

### For Banks & Financial Institutions
- Banks can integrate the POS contract as a **settlement layer**, issuing their own business tokens as stablecoins or loyalty points.
- The **bonding curve guarantees liquidity**, so banks don’t need to manage order books.
- Cross‑border payments settle in seconds with negligible fees.

### For Economies
- A network of millions of businesses, each with their own token, creates a **distributed, self‑regulating economy** where value flows through the main GNS bonding curve.
- The more businesses join, the more demand for GNS, creating a **virtuous cycle of adoption and price appreciation**.
- The entire system is **auditable on‑chain**, making it ideal for countries seeking transparent, programmable money.

---

## 🔐 Non‑Custodial & Decentralised

- No central entity controls the POS contract – it’s a Solana program deployed to a public network.
- Customers send SOL directly from their wallet; the merchant never holds the customer’s private key.
- The smart contract handles all minting, buying, and fee splits – trust is in the code, not a company.

---

## 📐 Economic Model

For a payment of `P` SOL:

1. **Protocol fee**: `f = FEE_LAMPORTS` (a constant small amount)
2. **Merchant share**: `M = P - f`
3. **Business tokens minted**: `T = bonding_curve_buy(business_curve, M)`
4. **Genesis fee tokens minted**: `G = bonding_curve_buy(main_curve, f)`
5. **Merchant GNS refund**: `G/2`
6. **Fee vault receives**: `G/2`

The business’s bonding curve **always provides liquidity** – the customer can later sell their tokens back for SOL at any time.  
The GNS refund incentivises merchants to join the Genesis ecosystem, as they earn a valuable asset with every sale.

---

## 🚀 Scaling to Millions of Businesses

Because every business token runs on its own bonding curve, the system scales linearly – new businesses add **new liquidity pools** without congesting a single market.  
The Blockkette wallet provides a unified UI, but the underlying transactions are all independent, so network capacity can handle millions of concurrent payments.

This is the **Visa/Mastercard of Web3**, but with:

- Zero chargebacks (payments are final)
- Global reach (no borders)
- Programmable loyalty (tokens can have utility beyond payments)
- Merchant sovereignty (no arbitrary account freezes)

---

## 📂 Where to look

- `contracts/genesis-pos/src/instructions/pos.rs` – The POS activation and payment processing logic.
- `contracts/genesis-business-factory/` – How businesses create their own tokens.
- `contracts/genesis-bonding-curve/` – The infinite liquidity engine that powers both the business token and GNS economies.
- `frontend/` – The Blockkette wallet that integrates all of this.

*Built by the Blockkette team. Testnet only.*
