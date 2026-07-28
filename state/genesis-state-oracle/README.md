# Oracle State Definitions

On‑chain accounts that store the **SOL/USD** and **GNS/USD** exchange rates, updated by an authorised oracle authority.  
Every price‑dependent instruction in the protocol reads these accounts.

---

## `SolUsdOracle`

Stored at PDA `[b"sol_usd_oracle"]`.

| Field | Type | Purpose |
|-------|------|---------|
| `oracle_authority` | Pubkey | Only this address can update the price |
| `price_usd_1e6` | u64 | SOL price in USD with 6‑decimal precision (e.g., $150.25 = 150_250_000) |
| `last_updated` | i64 | Unix timestamp of the last price update |
| `bump` | u8 | PDA bump seed |

Used by:
- **Bonding curve** – computing market cap in USD (for airdrop gating)
- **USDT swap** – converting SOL proceeds to USDT amount

---

## `GnsUsdOracle`

Stored at PDA `[b"gns_usd_oracle"]`.

| Field | Type | Purpose |
|-------|------|---------|
| `oracle_authority` | Pubkey | Only this address can update the price |
| `price_usd_1e6` | u64 | GNS price in USD with 6‑decimal precision |
| `last_updated` | i64 | Unix timestamp of the last price update |
| `bump` | u8 | PDA bump seed |

Used by:
- **Business Token Factory** – converting USD tier costs to GNS amounts
- **Perps** – converting PnL between USD and GNS
- **AI subscription** – converting USD subscription cost to GNS

---

## 🔒 Security

- Both oracles enforce a **staleness check** (`ORACLE_MAX_STALENESS_SECS`) – trades and claims revert if the price is too old.
- The `oracle_authority` can be updated by the protocol admin, allowing migration to a decentralised oracle network (Pyth, Switchboard) in the future.

---

*Built by the Blockkette team. Testnet only.*
