# Genesis Protocol Constants

**All economic, security, and operational constants used by the Genesis Anchor program.**  
These values define everything from token decimals and oracle staleness to fees, liquidation parameters, and governance timelocks.

---

## 🔢 Token & SOL Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `DECIMALS` | 9 | Number of decimals for GNS and business tokens |
| `ONE_TOKEN` | 10^9 = 1,000,000,000 | One whole token in base units |
| `LAMPORTS_PER_SOL` | 1,000,000,000 | Lamports in 1 SOL |
| `MIN_BASE_PRICE` | 10,000 | Minimum allowed base price for any bonding curve |
| `VAULT_RENT_EXEMPT_MIN` | 890,880 | Minimum lamports a vault must hold to remain rent‑exempt |

---

## 🏪 POS (Point‑of‑Sale)

| Constant | Value | Purpose |
|----------|-------|---------|
| `POS_FEE_LAMPORTS` | 10,000 | Protocol fee (in lamports) taken from each POS payment |

---

## 🔮 Oracle & Market Data

| Constant | Value | Purpose |
|----------|-------|---------|
| `ORACLE_MAX_STALENESS_SECS` | 90 seconds | Max age of an oracle price before trades/claims revert |

---

## ⚖️ Perps Risk Management

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAINTENANCE_MARGIN_BPS` | 1,000 (10%) | Minimum margin as a percentage of position value before liquidation |
| `LIQUIDATION_BOUNTY_GNS` | 10 GNS | Reward paid to liquidators for closing underwater positions |
| `MIN_NOTIONAL_USD_1E6` | 10,000 ($0.01) | Minimum notional value (in USD with 6 decimals) for an order to be valid |

---

## 💰 Fee Splits

| Constant | Value | Purpose |
|----------|-------|---------|
| `TREASURY_FEE_BPS` | 5,000 (50%) | Portion of protocol fees that go to the treasury |
| `STAKING_REWARD_BPS` | 5,000 (50%) | Portion of protocol fees distributed to GNS stakers |
| `CLOSE_INITIATOR_FEE_GNS` | 1,000,000 (0.001 GNS) | Fee paid to users who close their perps positions |

---

## 🛡 Governance & Safety

| Constant | Value | Purpose |
|----------|-------|---------|
| `CURVE_PARAM_TIMELOCK_SECS` | 86,400 (24 hours) | Delay before proposed bonding curve parameter changes take effect |
| `MAX_TRADE_FRACTION_BPS` | 100 (1%) | Maximum fraction of the total supply that can be traded in a single transaction |

---

## 👤 Initial Admin

| Constant | Value | Purpose |
|----------|-------|---------|
| `INITIAL_ADMIN_PUBKEY_STR` | `FCjRhDx4BtuTR86rmdgTw5cSAXJejbt7CvmGUP7rwWy2` | Public key that can initialise the ProtocolAdmin PDA (used exactly once) |

---

## 📂 Where to look

- `contracts/genesis-constants/src/constants.rs` – This file.
- `contracts/genesis-admin/` – Where `INITIAL_ADMIN_PUBKEY_STR` is used.
- `contracts/genesis-bonding-curve/` – Where `MIN_BASE_PRICE`, `VAULT_RENT_EXEMPT_MIN`, `CURVE_PARAM_TIMELOCK_SECS` are enforced.
- `contracts/genesis-perps/` – Where `MAINTENANCE_MARGIN_BPS`, `LIQUIDATION_BOUNTY_GNS`, `ORACLE_MAX_STALENESS_SECS` are applied.

*Built by the Blockkette team. Testnet only.*
