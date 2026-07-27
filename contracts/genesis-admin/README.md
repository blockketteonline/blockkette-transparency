# Protocol Admin – Governance Backbone

**The smart contract that manages the super‑admin key for the entire Genesis protocol – with a two‑step transfer process designed to eventually hand control to a DAO or multi‑sig.**

---

## 🛡 What Does the Protocol Admin Do?

The `ProtocolAdmin` account is the **highest authority** in the Genesis ecosystem. It controls:

- Initializing the `SolUsdOracle` and `GnsUsdOracle`
- Setting the `FactoryConfig` that governs all economic parameters
- Controlling the `paused` flag that can halt critical operations in an emergency
- Authorizing protocol upgrades and sensitive parameter changes

The admin is **not a person** – it’s a public key. Currently, it’s set at deployment to a hard‑coded initial address, but the contract is designed to transfer that power **securely** and **transparently**.

---

## 🔄 Two‑Step Admin Transfer

To prevent accidental loss of control or a single‑point‑of‑failure attack, the admin transfer follows a **two‑step process**:

1. **Nominate** – The current admin proposes a new admin (`pending_admin`).
2. **Accept** – The new admin must explicitly accept the role. Until acceptance, the current admin can cancel the nomination.

This prevents an attacker from forcing a malicious admin change even if they temporarily compromise the admin key – the new admin must actively confirm.

---

## 🏛 Path to Decentralization

The initial admin key will be held by the Blockkette team during testnet and early mainnet. **The ultimate goal is to transfer control to a DAO (governed by GNS token holders) or a multi‑signature wallet** controlled by community members and institutional partners.

The two‑step transfer system makes this transition seamless: nominate the DAO address, the DAO votes to accept, and control moves entirely on‑chain.

---

## 🔒 Security

- The admin key is checked in every privileged instruction (`constraint = protocol_admin.current_admin == admin.key()`).
- The `CancelAdminTransfer` instruction allows the current admin to revoke a nomination if the pending admin becomes compromised.
- The `InitializeProtocolAdmin` instruction can only be called by the hard‑coded initial admin, preventing anyone else from hijacking the protocol at deployment.

---

## 📂 Where to look

- `contracts/genesis-admin/src/instructions/protocol_admin.rs` – The admin initialization, nomination, acceptance, and cancellation logic.
- `contracts/genesis-admin/src/state/protocol_admin.rs` – The `ProtocolAdmin` account structure.

*Built by the Blockkette team. Testnet only.*
