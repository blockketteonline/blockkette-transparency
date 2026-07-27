# Genesis On‑Chain State Definitions

**The Solana account structures that store critical protocol data – airdrop escrows, claim statuses, and USDT swap configuration.**

---

## 🗄 Account Structures

### `AirdropEscrow`
Holds the parameters for a Merkle‑based airdrop distribution:
- `authority` – PDA that controls the escrowed tokens
- `mint` – The GNS token mint
- `escrow_token_account` – The token account holding the airdrop tokens
- `merkle_root` – Root hash of the Merkle tree of eligible claimants
- `total_amount` – Total tokens allocated to the airdrop
- `required_market_cap_usd_1e6` – Market cap threshold (in USD with 6 decimals) that must be reached before claims are allowed

### `AirdropClaimStatus`
A simple flag per user:
- `claimed` – Set to `true` once a user has successfully claimed their airdrop allocation (prevents double claiming)

### `UsdtSwapConfig`
Configuration for the GNS→USDT swap facility:
- `admin` – Authority that can update the config
- `usdt_mint` – The USDT token mint on Solana
- `usdt_vault` – The token account holding the USDT reserves
- `bump` – PDA bump seed for the config account itself
- `vault_authority_bump` – PDA bump seed for the USDT vault authority (used to sign transfers)

---

## 🔗 Where These Are Used

| State Struct | Used By |
|--------------|---------|
| `AirdropEscrow` | `airdrop.rs` – `init_airdrop` creates it, `claim_airdrop` reads it for merkle proof and market cap gating |
| `AirdropClaimStatus` | `airdrop.rs` – `claim_airdrop` sets `claimed = true` after a successful claim |
| `UsdtSwapConfig` | `usdt_swap.rs` – `init_usdt_swap_config` initializes it, `swap_gns_for_usdt` uses it to locate the USDT vault |

---

## 📂 Where to look

- `contracts/genesis-state/src/state.rs` – The account definitions (this file).
- `contracts/genesis-airdrop/` – The instruction that uses `AirdropEscrow` and `AirdropClaimStatus`.
- `contracts/genesis-usdt-swap/` – The instruction that uses `UsdtSwapConfig`.

*Built by the Blockkette team. Testnet only.*
