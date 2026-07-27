# Genesis Perpetuals (Perps)

**On‑chain perpetual futures trading with up to 100x leverage, powered by the Genesis Bonding Curve’s infinite liquidity.**

---

## 🧠 Architecture

The Perps system is a full‑featured perpetual futures exchange built directly into the Genesis Anchor program.  
Every trade is settled entirely on‑chain – no centralised order book, no counterparty risk.

### Core Components

| Account | Purpose |
|---------|---------|
| `PerpMarket` | Defines a trading market (e.g., BTC‑PERP) – symbol, max leverage, fees, oracle price |
| `PerpOrder` | A maker order placed by a user specifying side, margin, leverage, and limit price |
| `PerpPosition` | An open position created when a maker order is matched by a taker |
| `PerpUser` | A user’s margin account – tracks deposited and locked GNS |

### Trading Flow

1. **Create a market** – Admin initialises a `PerpMarket` with a symbol and risk parameters.
2. **Deposit collateral** – Users deposit GNS into their `PerpUser` account (via `perp_vault.rs`).
3. **Place an order** – A maker locks margin and creates a `PerpOrder` at a specific price.
4. **Take the order** – A taker matches the order, creating two mirrored `PerpPosition` accounts.
5. **Close position** – Either party can close their position; PnL is settled in GNS.
6. **Liquidate** – If a position’s loss exceeds the maintenance margin, anyone can liquidate it for a bounty.

---

## ♾️ Unlimited Liquidity from the Bonding Curve

The Perps system does not need external liquidity providers because it is backed by the **Genesis Bonding Curve**.

- GNS (the token traded on the bonding curve) is used as **margin** for all positions.
- Because the bonding curve **guarantees infinite buy/sell liquidity** for GNS, there is always a price and always an execution.
- When a user opens a position, they lock GNS – which effectively removes it from circulation, increasing buying pressure on the bonding curve.
- When a user closes with a profit, new GNS is minted; with a loss, GNS is burned. Both operations interact directly with the curve’s supply, keeping the entire system self‑balancing.

This means the Perps exchange **can scale to any number of markets** without worrying about liquidity fragmentation. Every asset pair (crypto, stocks, indices, commodities) can be supported as long as an oracle price feed exists.

---

## 🌍 Multi‑Asset Capability

The `PerpMarket` struct uses a `symbol` field (`[u8; 16]`), allowing the protocol to create markets for:

- Crypto pairs: BTC‑PERP, ETH‑PERP, SOL‑PERP, HYPE‑PERP
- Traditional assets: AAPL‑PERP, TSLA‑PERP, SPX‑PERP (via oracle integration)
- Forex, commodities, and more

Every market uses **GNS as margin**, tying all trading activity back to the Genesis ecosystem.

---

## ⚖️ Risk Management

- **Max leverage** – configurable per market (up to 100x).
- **Price deviation check** – an order can only be matched if the limit price is within a configurable percentage of the oracle price.
- **Oracle staleness check** – trades revert if the price hasn’t been updated recently.
- **Maintenance margin** – positions must maintain a minimum margin or face liquidation.
- **Liquidation bounty** – incentivises keepers to liquidate underwater positions.

---

## 📐 Mathematics

### Position PnL Calculation

For a long position:
\[
\text{PnL}_{\text{USD}} = \text{Margin}_{\text{GNS}} \times \frac{\text{Price}_{\text{exit}} - \text{Price}_{\text{entry}}}{\text{Price}_{\text{entry}}} \times \text{Leverage}
\]

For a short position:
\[
\text{PnL}_{\text{USD}} = \text{Margin}_{\text{GNS}} \times \frac{\text{Price}_{\text{entry}} - \text{Price}_{\text{exit}}}{\text{Price}_{\text{entry}}} \times \text{Leverage}
\]

The USD PnL is then converted to GNS using the on‑chain `GnsUsdOracle`.

---

## 🔒 Security

- All margin is held in program‑derived accounts (PDAs) – the protocol never has custody.
- The `PerpMarket` can be paused by the Factory Config in an emergency.
- Oracle prices are validated for staleness before every trade.

---

## 📂 Where to look

- `contracts/genesis-perps/src/instructions/perp_market.rs` – The core perps instruction (create market, place/take order, close, liquidate).
- `contracts/genesis-perps/src/instructions/perp_vault.rs` – Deposit/withdraw collateral.
- `contracts/genesis-perps/src/math.rs` – PnL computation and currency conversion.
- `contracts/genesis-perps/src/state/perps.rs` – Account structures.

*Built by the Blockkette team. Testnet only.*
