# Genesis Factory Config – Protocol Governance

This smart contract holds the **global configuration** of the Genesis ecosystem. It’s the economic rulebook for the Business Token Factory, AI subscriptions, and Perps – all stored on‑chain and auditable by anyone.

## What it controls

| Parameter | Purpose | Affects |
|-----------|---------|---------|
| `default_base_price` | Starting price for new business tokens | All business tokens created |
| `default_price_increment` | Slope of business token bonding curves | Token price dynamics |
| `tiers` | Array of pricing tiers (USD cost, token supply) | Business token creation/upgrade |
| `gns_usd_price_cents` | GNS/USD oracle price (1e2) | Conversion of USD fees to GNS |
| `pos_activation_cost_gns` | GNS fee to open a perps position | Perps trading |
| `ai_subscription_cost_gns` | GNS fee for AI Pro subscription | AI access |
| `ai_subscription_period_secs` | Duration of AI subscription | Renewal cycles |
| `paused` | Emergency stop for all critical operations | Safety |

## Admin vs. Community

The `admin` key is the only one allowed to call `update_factory_config`.  
**But this is not a permanent dictatorship:** the admin can be changed to a multi‑sig or DAO, and all parameter updates are visible on‑chain.  

The system is designed so that once the protocol matures, **governance will transition to GNS token holders** – making the factory truly community‑controlled.

## Why on‑chain?

- **Verifiable:** anyone can read the current config and confirm fees haven’t been changed secretly.
- **Immutable rules:** businesses know exactly what they pay to create tokens.
- **Trust minimized:** no need to trust a website; the contract is the source of truth.

## 🔗 Integration with other modules

- `business_token.rs` reads `config.tiers` to determine token supply and fees.
- `airdrop.rs` reads `config.paused` to block claims during emergencies.
- `bonding_curve.rs` uses `config.paused` to halt trades if needed.
- `subscription.rs` uses `ai_subscription_cost_gns` and `ai_subscription_period_secs`.

## 📂 Where to look

- `contracts/genesis-factory-config/src/instructions/factory.rs` – The config management code.
- `contracts/genesis-factory-config/src/state/factory.rs` – The `FactoryConfig` account struct.

*Built by the Blockkette team. Testnet only.*
