# Genesis Anchor Program – Entry Point

**The main `lib.rs` that ties together every module, declares the on‑chain program ID, and exposes all 30+ instructions that clients can call.**

---

## 🧩 What This File Does

- **Declares the program ID** via `declare_id!("…")` – this is the on‑chain address of the deployed Genesis program.
- **Imports and re‑exports all sub‑modules** – `constants`, `errors`, `events`, `math`, `state`, `instructions`.
- **Defines the `#[program]` module** – each public function is an **entry point** that the Solana runtime can call.
- **Routes every instruction** to its corresponding handler in `instructions::*`.

---

## 📡 Instruction List (what clients can call)

| Instruction | Category | Purpose |
|-------------|----------|---------|
| `initialize_protocol_admin` | Admin | Create the protocol admin PDA |
| `nominate_protocol_admin` | Admin | Propose a new admin |
| `accept_protocol_admin` | Admin | Accept admin transfer |
| `cancel_protocol_admin_transfer` | Admin | Cancel a pending admin transfer |
| `initialize` | Bonding Curve | Deploy a new bonding curve |
| `buy` | Bonding Curve | Buy tokens from the curve |
| `sell` | Bonding Curve | Sell tokens to the curve |
| `nominate_admin` | Bonding Curve | Propose curve admin transfer |
| `accept_admin` | Bonding Curve | Accept curve admin transfer |
| `cancel_admin_transfer` | Bonding Curve | Cancel curve admin transfer |
| `update_curve_params` | Bonding Curve | Propose new curve parameters (timelocked) |
| `execute_curve_params_update` | Bonding Curve | Apply timelocked curve params |
| `cancel_curve_params_update` | Bonding Curve | Cancel pending params |
| `initialize_factory_config` | Factory | Set global economic config |
| `update_factory_config` | Factory | Update global config |
| `pause_protocol` / `unpause_protocol` | Factory | Emergency pause/unpause |
| `create_business_token` | Business | Create a new business token |
| `upgrade_business_token` | Business | Upgrade to higher tier |
| `update_business_curve_params` | Business | Adjust business token's bonding curve |
| `activate_pos` | POS | Activate merchant Point‑of‑Sale |
| `process_pos_payment` | POS | Customer pays → merchant gets tokens |
| `purchase_ai_subscription` | Subscription | Pay GNS to unlock AI Pro |
| `initialize_perp_vault` | Perps | Create shared GNS vault |
| `perp_deposit` / `perp_withdraw` | Perps | Deposit/withdraw margin |
| `init_staking_pool` | Staking | Create staking pool |
| `stake_gns` / `unstake_gns` | Staking | Stake/unstake GNS |
| `sweep_donations` | Staking | Move excess GNS to treasury |
| `initialize_sol_usd_oracle` | Oracle | Create SOL/USD price feed |
| `update_sol_usd_price` | Oracle | Update SOL/USD price |
| `initialize_gns_usd_oracle` | Oracle | Create GNS/USD price feed |
| `update_gns_usd_price` | Oracle | Update GNS/USD price |
| `create_perp_market` | Perps | Create a trading pair |
| `create_perp_order` | Perps | Place a maker order |
| `take_perp_order` | Perps | Match a maker order |
| `close_perp_position` | Perps | Close an open position |
| `liquidate_position` | Perps | Liquidate an underwater position |
| `init_airdrop` | Airdrop | Set up Merkle‑tree airdrop |
| `claim_airdrop` | Airdrop | Claim airdrop (market‑cap gated) |
| `init_usdt_swap_config` | Swap | Configure USDT swap facility |
| `swap_gns_for_usdt` | Swap | Burn GNS → receive USDT |

---

## 🔗 Integration with Frontend

The frontend’s `genesisProgram.ts` (and the IDL at `src/idl/genesis_token.json`) are generated from this file.  
Every instruction listed here corresponds to a TypeScript method that the Blockkette wallet can call.

---

## 📂 Where to look

- `contracts/genesis-program/src/lib.rs` – This file.
- `contracts/genesis-bonding-curve/`, `contracts/genesis-perps/`, etc. – The individual instruction modules that `lib.rs` routes to.
- `contracts/genesis-constants/`, `contracts/genesis-errors/`, `contracts/genesis-events/`, `contracts/genesis-state/` – The supporting modules imported at the top.

*Built by the Blockkette team. Testnet only.*
