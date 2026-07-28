# Staking State Definitions

On‑chain accounts that track **GNS staking** – the pool that holds staked tokens and each user's proportional share.

---

## `StakingPool`

The global staking pool. Stored at PDA `[b"staking_pool"]`.

| Field | Type | Purpose |
|-------|------|---------|
| `gns_mint` | Pubkey | The GNS token mint |
| `pool_ata` | Pubkey | The token account owned by the staking pool PDA |
| `total_shares` | u64 | Total shares issued (represents proportional ownership of the pool) |
| `total_staked` | u64 | Total GNS currently staked in the pool |
| `bump` | u8 | PDA bump seed |

The pool can receive donations (e.g., protocol fees). Because `total_shares` is fixed when no one stakes/unstakes, donations **increase the value of each share** – this is how stakers earn yield.

---

## `StakeRecord`

A user's individual staking position. Stored at PDA `[b"stake_record", user.key().as_ref()]`.

| Field | Type | Purpose |
|-------|------|---------|
| `owner` | Pubkey | The staker's wallet |
| `share_balance` | u64 | Number of shares the user owns |

---

## 📐 Mathematics

When staking `amount` GNS:
\[
\text{shares} = \frac{\text{amount} \times \text{total\_shares}}{\text{total\_staked}}
\]

When unstaking `share_amount` shares:
\[
\text{gns\_returned} = \frac{\text{share\_amount} \times \text{total\_staked}}{\text{total\_shares}}
\]

If the pool has received donations, `total_staked` grows while `total_shares` stays the same – so each share becomes worth more GNS over time.

---

## 🔗 Future Governance

The `share_balance` in `StakeRecord` will serve as **voting power** in the Genesis DAO, letting long‑term stakers decide on protocol parameters, fee distribution, and upgrades.

---

*Built by the Blockkette team. Testnet only.*
