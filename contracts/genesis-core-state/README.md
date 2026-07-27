# Genesis Core State – The Heart of the Protocol

**The two most critical on‑chain accounts: the `BondingCurve` (every buy/sell/price calculation depends on it) and the `ProtocolAdmin` (the governance key).**

---

## 📊 `BondingCurve`

This single account holds **all the data that defines a Genesis bonding curve**:

| Field | Type | Purpose |
|-------|------|---------|
| `admin` | Pubkey | Address that can update curve parameters |
| `pending_admin` | Pubkey | New admin nominated (two‑step transfer) |
| `token_mint` | Pubkey | The SPL token mint for this curve's token |
| `sol_vault` | Pubkey | The SOL vault that backs the curve's liquidity |
| `base_price` | u64 | Starting price when supply = 0 |
| `price_increment` | u64 | Slope – price increase per token minted |
| `total_supply` | u64 | Current circulating supply of the token |
| `max_supply` | u64 | Hard cap on token supply |
| `total_sol_raised` | u64 | Total SOL deposited into the curve since inception |
| `pending_base_price` | u64 | Proposed new base price (timelocked) |
| `pending_price_increment` | u64 | Proposed new price increment (timelocked) |
| `pending_max_supply` | u64 | Proposed new max supply (timelocked) |
| `pending_timestamp` | i64 | Unix timestamp when pending params can be executed |
| `last_trade_slot` | u64 | Solana slot of the last trade (cooldown enforcement) |

Every buy, sell, price quote, and airdrop market‑cap check reads from this account. It is the **source of truth** for the token's economic state.

---

## 🛡 `ProtocolAdmin`

A lightweight account that governs the entire protocol:

| Field | Type | Purpose |
|-------|------|---------|
| `current_admin` | Pubkey | The active admin key (can be a multi‑sig or DAO) |
| `pending_admin` | Pubkey | Admin nominated for transfer (two‑step acceptance) |
| `bump` | u8 | PDA bump seed |

Every privileged instruction (factory init, oracle init, pause/unpause) checks that the signer matches `current_admin`. The two‑step transfer prevents accidental loss of control.

---

## 🔗 Where These Are Used

- `BondingCurve` is referenced by **every** instruction that touches token economics: `buy`, `sell`, `airdrop`, `usdt_swap`, `pos_payment`, and more.
- `ProtocolAdmin` is checked by `initialize_protocol_admin`, `init_airdrop`, `init_factory_config`, `init_sol_usd_oracle`, and other admin‑gated instructions.

---

## 📂 Where to look

- `contracts/genesis-core-state/src/core_state.rs` – The struct definitions (this file).
- `contracts/genesis-bonding-curve/` – The buy/sell instructions that mutate `BondingCurve`.
- `contracts/genesis-admin/` – The instructions that manage `ProtocolAdmin`.

*Built by the Blockkette team. Testnet only.*
