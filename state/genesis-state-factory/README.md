# Factory & Business State Definitions

On‑chain account structures that govern the protocol’s economic configuration, merchant point‑of‑sale, and AI subscription status.

---

## `FactoryConfig`

The global configuration that controls all economic parameters. Stored at PDA `[b"factory_config"]`.

| Field | Type | Purpose |
|-------|------|---------|
| `admin` | Pubkey | Authority that can update the config |
| `gns_mint` | Pubkey | The GNS token mint |
| `fee_vault` | Pubkey | Token account that collects protocol fees |
| `gns_usd_price_cents` | u64 | GNS/USD price in cents (1/100 of a USD) – used for USD → GNS conversion |
| `default_base_price` | u64 | Default starting price for new business tokens |
| `default_price_increment` | u64 | Default slope for business token bonding curves |
| `pos_activation_cost_gns` | u64 | GNS fee to activate a merchant POS |
| `ai_subscription_cost_gns` | u64 | GNS fee for AI Pro subscription |
| `ai_subscription_period_secs` | i64 | Duration of AI subscription in seconds |
| `tiers` | [Tier; 10] | Up to 10 pricing tiers for business token creation/upgrade |
| `paused` | bool | Emergency stop flag |

---

## `Tier`

Describes a single pricing tier for the Business Token Factory.

| Field | Type | Purpose |
|-------|------|---------|
| `usd_cost` | u64 | Cost of this tier in USD (converted to GNS at the current oracle price) |
| `token_supply` | u64 | Maximum token supply the business gets at this tier |

---

## `MerchantPOS`

Tracks a merchant’s point‑of‑sale activation status. Stored at PDA `[b"merchant_pos", merchant.key().as_ref()]`.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The merchant's wallet |
| `business_token_mint` | Pubkey | The mint of the business token this POS accepts |
| `activated` | bool | Whether the POS is active |

---

## `AISubscription`

Records a user’s AI subscription expiry. Stored at PDA `[b"ai_subscription", user.key().as_ref()]`.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The subscriber's wallet |
| `expiry` | i64 | Unix timestamp when the subscription ends |

---

## 🔗 Where These Are Used

- `FactoryConfig` – read by factory init, business token creation, AI subscription purchase, airdrop claims, POS operations.
- `Tier` – part of `FactoryConfig`, used by `create_business_token` and `upgrade_business_token`.
- `MerchantPOS` – created by `activate_pos`, checked by `process_pos_payment`.
- `AISubscription` – created/updated by `purchase_ai_subscription`, read by frontend to gate AI features.

---

*Built by the Blockkette team. Testnet only.*
