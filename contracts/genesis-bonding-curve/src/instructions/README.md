# Instruction Modules – Genesis Bonding Curve Program

This directory contains every on‑chain operation the Genesis smart contract can perform.  
Each file is a self‑contained Anchor instruction module, and `mod.rs` re‑exports them all.

## 📁 Instruction Files

| File | Purpose |
|------|---------|
| `bonding_curve.rs` | Core buy/sell on the linear bonding curve, admin transfer, curve parameter updates |
| `factory.rs` | Global protocol configuration (tiers, fees, pauses) |
| `business_token.rs` | Business Token Factory – create, upgrade, and configure custom tokens |
| `airdrop.rs` | Merkle‑tree airdrop initialization and market‑cap‑gated claims |
| `pos.rs` | Perpetuals position management (open, close, liquidate) |
| `perp_vault.rs` | Perps vault deposits and withdrawals |
| `perp_market.rs` | Perps market creation and configuration |
| `staking.rs` | GNS staking for rewards and governance weight |
| `subscription.rs` | Token‑gated subscription management for AI and pro features |
| `oracle.rs` | On‑chain SOL/USD oracle updates |
| `usdt_swap.rs` | USDT swap integration (stablecoin entry/exit) |
| `protocol_admin.rs` | Protocol admin management (multi‑sig ready) |

## 🔗 How They Work Together

- The **bonding curve** is the central liquidity engine – every business token, every perps trade, and every subscription fee interacts with it.
- **Factory config** sets the economic parameters that all other modules read.
- **Airdrop** locks tokens until the market cap reaches a target – aligning community incentives.
- **Business tokens** pay GNS fees that flow back into the bonding curve, creating sustainable demand.

All instructions are **public and auditable** – the `mod.rs` makes them available under a single namespace for the client libraries (like `genesisProgram.ts` in the frontend).

*Built by the Blockkette team. Testnet only.*
