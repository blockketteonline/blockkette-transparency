# Genesis AI Subscription – Token‑Gated Intelligence

**The on‑chain purchase record that unlocks AI Pro features in the Blockkette wallet – paid in GNS, auditable, and stacking for long‑term users.**

---

## 🧠 What It Does

The `purchase_ai_subscription` instruction allows a user to pay the **current GNS subscription fee** (set in `FactoryConfig`) and extend their **AI subscription expiry** on‑chain.

- The fee is transferred from the user's GNS token account to the protocol's fee vault.
- The user's `AISubscription` account stores the `owner` and `expiry` timestamp.
- If the user already has an active subscription, the new period is **added to the existing expiry** – subscriptions stack.

---

## 🔗 Integration with the AI Advisor

The Blockkette AI Advisor (`ai_advisor.py` + `AIAdvisorPanel.tsx`) provides real‑time 4H trading signals.  
The frontend reads the `AISubscription` account on‑chain to determine if the user has an active subscription.

- **Free tier:** basic indicators, 1 AI call per hour.
- **Pro tier (subscription active):** unlimited calls, all three strategies, position sizing – **unlocked only when `expiry > current_time`**.

This creates a **direct, on‑chain gating mechanism** – users must hold GNS and pay the subscription fee to access premium intelligence.

---

## 📐 Economic Model

The subscription fee (`ai_subscription_cost_gns`) and period (`ai_subscription_period_secs`) are stored in the `FactoryConfig`, which is updated by governance.

- **Revenue flows to the fee vault** – a portion of these fees can later be distributed to GNS stakers via the staking pool.
- **GNS demand increases** as more users subscribe to AI Pro – every new subscriber buys GNS from the open market, driving value back to the bonding curve.

---

## 🔒 Security & Transparency

- The subscription expiry is **on‑chain and publicly readable** – no server‑side secrets.
- The `FactoryConfig.paused` flag can pause new purchases in an emergency, but existing subscriptions are unaffected.
- All payments are recorded as Solana transactions, fully auditable.

---

## 📂 Where to look

- `contracts/genesis-subscription/src/instructions/subscription.rs` – The subscription purchase logic.
- `contracts/genesis-factory-config/` – Where `ai_subscription_cost_gns` and `ai_subscription_period_secs` are set.
- `backend/app/routers/ai_advisor.py` – The AI engine gated by this subscription.

*Built by the Blockkette team. Testnet only.*
