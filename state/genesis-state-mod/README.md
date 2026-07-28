# Genesis State Module Index

This file (`mod.rs`) is the **entry point for all on‑chain account structures** used by the Genesis Anchor program.  
It declares every state sub‑module and re‑exports them so they are accessible from a single import path.

---

## 📁 Sub‑modules

| Sub‑module | Purpose |
|------------|---------|
| `bonding_curve` | The `BondingCurve` account – holds all data for the bonding curve (price, supply, vault, admin) |
| `factory` | The `FactoryConfig`, `Tier`, `MerchantPOS`, and `AISubscription` accounts – global economic config and business state |
| `perps` | The `PerpMarket`, `PerpOrder`, `PerpPosition`, and `PerpUser` accounts – perpetual futures trading |
| `staking` | The `StakingPool` and `StakeRecord` accounts – GNS staking for rewards and governance |
| `oracle` | The `SolUsdOracle` and `GnsUsdOracle` accounts – on‑chain price feeds |
| `airdrop` | The `AirdropEscrow` and `AirdropClaimStatus` accounts – Merkle‑tree airdrop state |

---

## 🔗 Why a Separate Module Index?

In Rust/Anchor programs, every state struct must be declared in a module that is publicly accessible. This `mod.rs` centralizes all state definitions so that:

- Instruction files can import state with `use crate::state::*;`
- The Anchor framework can properly deserialize account data
- All account sizes and constraints are defined in one logical place

---

## 📂 Where the actual structs live

Each sub‑module is defined in its own file inside the `state/` directory of the Anchor program:
*Built by the Blockkette team. Testnet only.*
