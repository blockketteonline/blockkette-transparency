# Perp Vault – Self‑Custodial Collateral Engine

**The on‑chain vault that holds GNS margin for perpetual futures trading – fully non‑custodial, auditable, and integrated with the Genesis Bonding Curve.**

---

## 🏦 What Is the Perp Vault?

The Perp Vault is a **program‑derived account (PDA)** that acts as the shared collateral pool for all perps traders. Instead of each user having an isolated wallet, all margin deposits flow into a single vault controlled entirely by the smart contract.

This design provides:
- **Atomic settlement** – profits and losses are applied instantly within the program.
- **Gas efficiency** – a single pool avoids scattered token accounts.
- **Liquidity sharing** – margin from all users backs the global perps market.

---

## 🔒 How It Works

### Deposit
1. User transfers GNS from their wallet to the vault’s ATA.
2. The vault increases the user’s `deposited` balance in their `PerpUser` account.
3. The GNS is now available as **free margin** to place orders.

### Withdraw
1. User requests withdrawal of GNS.
2. The contract checks that the user has enough **free margin** (`deposited - locked_margin`).
3. The vault PDA signs a token transfer back to the user’s wallet.

### Locked vs. Free Margin
- **Deposited** – total GNS the user has placed into the vault.
- **Locked** – GNS currently tied up in open orders or positions.
- **Free** – GNS available to withdraw or use for new trades.

This separation ensures that users cannot withdraw collateral that is backing active positions – protecting counterparties.

---

## 🛡 Security & Uniqueness

- **Non‑custodial** – the vault is a PDA, meaning only the program can move funds. No admin key can steal GNS.
- **Emergency pause** – the Factory Config can pause deposits/withdrawals in a critical situation.
- **Transparent** – all balances are on‑chain and auditable.
- **Integrated with the bonding curve** – GNS in the vault is effectively removed from circulation, increasing scarcity and supporting the curve’s price.

Unlike centralised exchanges that hold user funds in opaque accounts, the Genesis Perp Vault provides **mathematical, verifiable proof of solvency** at every block.

---

## 📂 Where to look

- `contracts/genesis-perps/src/instructions/perp_vault.rs` – The vault initialization, deposit, and withdrawal logic.
- `contracts/genesis-perps/src/instructions/perp_market.rs` – How deposited margin is used in trades.
- `contracts/genesis-perps/src/state/perps.rs` – The `PerpUser` and `PerpVault` account structures.

*Built by the Blockkette team. Testnet only.*
