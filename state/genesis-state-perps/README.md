# Perps State Definitions

On‑chain accounts that store **margin, orders, open positions, and market configuration** for the Genesis perpetual futures exchange.

---

## `PerpVault`

The shared vault that holds all GNS collateral for perps trading. Stored at PDA `[b"perp_vault"]`.

| Field | Type | Purpose |
|-------|------|---------|
| `gns_mint` | Pubkey | The GNS token mint |
| `vault_ata` | Pubkey | The token account owned by the vault PDA |
| `bump` | u8 | PDA bump seed |

---

## `PerpUser`

Each trader's margin account. Stored at PDA `[b"perp_user", user.key().as_ref()]`.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The trader's wallet |
| `deposited` | u64 | Total GNS deposited into the vault |
| `locked_margin` | u64 | GNS currently locked in open orders or positions |
| `position_nonce` | u64 | Incremented each time a position is opened – ensures unique position PDAs |

**Free margin** = `deposited - locked_margin`. Users can only withdraw or open new orders with free margin.

---

## `PerpMarket`

Configuration for a specific perps trading pair (e.g., BTC‑PERP). Stored at PDA `[b"perp_market", symbol]`.

| Field | Type | Purpose |
|-------|------|---------|
| `admin` | Pubkey | Market creator |
| `symbol` | [u8; 16] | Ticker symbol (e.g., b"BTC‑PERP") |
| `oracle_authority` | Pubkey | Only this key can update the market price |
| `price_usd_1e6` | u64 | Current index price (6‑decimal precision) |
| `last_updated` | i64 | Timestamp of last price update |
| `max_leverage` | u16 | Maximum leverage (e.g., 100 = 100x) |
| `taker_fee_bps` | u16 | Fee in basis points (e.g., 10 = 0.1%) |
| `max_deviation_bps` | u16 | Max allowed deviation between order price and oracle (in bps) |
| `active` | bool | Whether trading is enabled |
| `open_interest_long_gns` | u64 | Total GNS locked in long positions |
| `open_interest_short_gns` | u64 | Total GNS locked in short positions |
| `bump` | u8 | PDA bump seed |
| `price_delay_secs` | i64 | Delay before a new price becomes active (anti‑frontrunning) |
| `pending_price_usd_1e6` | u64 | Price queued for activation |
| `pending_price_ts` | i64 | Timestamp when the pending price becomes active |

---

## `PerpOrder`

A maker order placed by a user. Stored at a unique PDA derived from owner, market, side, margin, leverage, and limit price.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The order creator |
| `market` | Pubkey | The PerpMarket this order targets |
| `side` | u8 | 0 = long, 1 = short |
| `margin_gns` | u64 | GNS margin locked for this order |
| `leverage` | u16 | Chosen leverage |
| `price_usd_1e6` | u64 | Limit price (order only matches at this price or better) |
| `bump` | u8 | PDA bump seed |
| `matched` | bool | Whether the order has been taken by a taker |

When matched, the order is **not closed** – the `matched` flag prevents double‑matching, and two `PerpPosition` accounts are created.

---

## `PerpPosition`

An open position created when a maker order is matched. Stored at PDA `[b"perp_position", owner, market, nonce]`.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The position owner |
| `market` | Pubkey | The PerpMarket |
| `side` | u8 | 0 = long, 1 = short |
| `margin_gns` | u64 | GNS margin backing this position |
| `leverage` | u16 | Position leverage |
| `entry_price_usd_1e6` | u64 | Entry price (from order) |
| `entry_gns_usd_1e6` | u64 | GNS/USD price at entry (for PnL calculation) |
| `notional_usd_1e6` | u128 | Total position value = margin × leverage × GNS price |
| `tp_price_usd_1e6` | u64 | Take‑profit price (user‑set) |
| `sl_price_usd_1e6` | u64 | Stop‑loss price (user‑set) |
| `opened_at` | i64 | Unix timestamp when opened |
| `bump` | u8 | PDA bump seed |
| `counterparty` | Pubkey | The taker's wallet |
| `counterparty_position` | Pubkey | The taker's corresponding position account |
| `nonce` | u64 | Unique nonce for this position |

**PnL calculation** uses `entry_price_usd_1e6` vs. current market price, scaled by `margin_gns` and `leverage`, then converted to GNS via `entry_gns_usd_1e6` and the current `GnsUsdOracle`.

---

## 🔗 Where These Are Used

- `PerpVault` – `initialize_perp_vault`, `perp_deposit`, `perp_withdraw`
- `PerpUser` – `perp_deposit`, `perp_withdraw`, `create_perp_order`, `take_perp_order`, `close_perp_position`, `liquidate_position`
- `PerpMarket` – `create_perp_market`, `create_perp_order`, `take_perp_order`, `close_perp_position`, `liquidate_position`
- `PerpOrder` – `create_perp_order`, `take_perp_order`
- `PerpPosition` – `take_perp_order`, `close_perp_position`, `liquidate_position`

---

*Built by the Blockkette team. Testnet only.*
