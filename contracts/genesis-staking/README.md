# Genesis Staking – GNS Rewards & Governance

**A non‑custodial staking pool that lets GNS holders earn shares of protocol fees and eventually vote on governance proposals.**

---

## 🏊 How It Works

1. **Initialize the staking pool** – A single `StakingPool` account is created, holding all staked GNS.
2. **Stake GNS** – Users transfer GNS to the pool and receive **shares** representing their proportional ownership.
3. **Unstake GNS** – Users burn shares and receive GNS back based on the pool's current value (including any accumulated rewards).
4. **Sweep Donations** – Any excess GNS in the pool (from protocol fees, airdrops, or donations) can be moved to the treasury, increasing the pool's value per share.

---

## 📐 Share Calculation

Shares are minted in proportion to the pool's existing balance:

\[
\text{shares} = \frac{\text{amount} \times \text{total\_shares}}{\text{total\_staked}}
\]

When unstaking, the returned GNS is calculated as:

\[
\text{gns\_returned} = \frac{\text{shares\_burned} \times \text{total\_staked}}{\text{total\_shares}}
\]

Because the pool can receive donations (via `sweep_donations`), the **ratio of total_staked to total_shares can increase over time**, meaning each share becomes worth more GNS – this is how stakers earn yield.

---

## 🗳 Governance (Future)

The `StakeRecord` account tracks each user's share balance. In the future, this stake will be used as **voting power** in a DAO, allowing GNS holders to vote on:

- Protocol parameter changes (fees, tier pricing, curve parameters)
- Treasury allocation
- Protocol upgrades

The more GNS you stake, the more influence you have.

---

## 🔗 Integration with the Ecosystem

- **Fee Vault** – A portion of protocol fees (from business token creation, POS transactions, AI subscriptions) will be routed to the staking pool, rewarding long‑term GNS holders.
- **Airdrop** – Initial community members who stake their airdropped GNS will earn additional rewards, incentivizing holding over selling.
- **Perps** – Users who stake GNS may receive reduced fees or higher leverage limits, making staking valuable for active traders.

---

## 🔒 Security

- The staking pool is a PDA – only the program can move staked funds.
- The `sweep_donations` function can only be called by the protocol admin, preventing unauthorized withdrawal of excess tokens.
- `EmergencyPaused` check prevents staking/unstaking during protocol emergencies.

---

## 📂 Where to look

- `contracts/genesis-staking/src/instructions/staking.rs` – The staking, unstaking, and sweep logic.
- `contracts/genesis-staking/src/state/staking.rs` – The `StakingPool` and `StakeRecord` account structures.

*Built by the Blockkette team. Testnet only.*
