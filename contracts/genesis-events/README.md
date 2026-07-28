# Genesis Protocol Events

All **structured events** emitted by the Genesis Anchor program. These are stored on‑chain in transaction logs and can be indexed by front‑ends, analytics dashboards, and off‑chain services.

---

## 🧾 Event Reference

| Event | Fields | Emitted When |
|-------|--------|--------------|
| `CurveInitialized` | `admin, token_mint, sol_vault, base_price, price_increment, max_supply` | A new bonding curve is deployed |
| `AdminUpdated` | `old_admin, new_admin` | A bonding curve admin accepts a transfer |
| `CurveParamsUpdated` | `admin, base_price, price_increment, max_supply` | Bonding curve parameters are changed (after timelock) |
| `BuyEvent` | `buyer, sol_amount, tokens_minted, new_total_supply, current_price` | A buy is executed on the bonding curve |
| `SellEvent` | `seller, tokens_burned, sol_returned, new_total_supply, current_price` | A sell is executed on the bonding curve |
| `BusinessTokenCreated` | `creator, token_mint, max_supply, tier_usd_cost` | A new business token is created via the Factory |
| `BusinessTokenUpgraded` | `owner, token_mint, new_max_supply, additional_cost_usd` | A business token is upgraded to a higher tier |
| `POSActivated` | `merchant, business_token_mint` | A merchant activates their Point‑of‑Sale |
| `POSPayment` | `customer, merchant, business_token_mint, sol_paid, tokens_received, fee_sol, genesis_fee, merchant_genesis_refund` | A customer pays via POS |
| `AISubscriptionPurchased` | `user, expiry` | A user extends their AI subscription |
| `PerpDepositEvent` | `user, amount` | GNS is deposited into the perps vault |
| `PerpWithdrawEvent` | `user, amount` | GNS is withdrawn from the perps vault |
| `PerpMarketCreated` | `market, symbol, max_leverage, taker_fee_bps, max_deviation_bps, price_delay_secs` | A new perps market is created |
| `PriceUpdated` | `market, price_usd_1e6, is_sol_usd` | A perps market price is updated |
| `GnsPriceUpdated` | `price_usd_1e6` | The GNS/USD oracle price is updated |
| `OrderPlaced` | `owner, side, margin_gns, leverage, price_usd_1e6` | A maker order is placed |
| `OrderMatched` | `maker, taker, price_usd_1e6, margin_gns, leverage` | A maker order is matched by a taker |
| `PositionOpened` | `owner, market, side, margin_gns, leverage, entry_price_usd_1e6` | A perps position is opened |
| `PositionClosed` | `owner, market, payout_gns, margin_gns, profit_gns, loss_gns` | A perps position is closed with PnL |
| `PositionLiquidated` | `owner, market, liquidator, payout_gns` | A perps position is liquidated |
| `Staked` | `user, shares, gns_amount` | GNS is staked |
| `Unstaked` | `user, shares, gns_amount` | GNS is unstaked |
| `AirdropInitialized` | `amount, merkle_root` | An airdrop is set up |
| `AirdropClaimed` | `claimant, amount` | A user successfully claims an airdrop |
| `GnsForUsdtSwap` | `user, gns_burned, usdt_received, sol_equivalent` | A GNS→USDT swap is executed |

---

## 📡 Usage

The frontend and backend listen to these events to:

- Update wallet balances and portfolio charts in real time
- Track trade history, PnL, and subscription status
- Index airdrop claims and staking activity
- Monitor protocol health (e.g., total supply, SOL raised, open interest)

All events are **immutable and verifiable** – they are part of the transaction history on Solana.

---

## 📂 Where to look

- `contracts/genesis-events/src/events.rs` – This file.
- Every instruction file (`bonding_curve.rs`, `perp_market.rs`, `airdrop.rs`, …) emits at least one of these events.

*Built by the Blockkette team. Testnet only.*
