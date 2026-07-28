# Genesis Math Library

**The low‑level Rust implementation of every arithmetic operation used by the Genesis Anchor program.**  
This file contains the bonding‑curve integrals, U256 big‑integer helpers, PnL calculations, market‑cap formulas, and Merkle‑proof verification – all written with checked math to prevent overflows and ensure deterministic execution.

---

## 🔢 Core Functions

### `get_current_price(state: &BondingCurve) -> u64`
Computes the current token price in lamports (SOL) using the linear formula:
\[
\text{price} = \text{base\_price} + \text{price\_increment} \times \left\lfloor \frac{\text{total\_supply}}{\text{ONE\_TOKEN}} \right\rfloor
\]

### `bonding_curve_buy(state, sol_amount) -> tokens_minted`
Calculates how many tokens are minted when a user deposits `sol_amount` lamports.  
For a linear price curve \( P(S) = \text{base} + \text{inc} \cdot S \), the cost function is the integral:
\[
\text{cost} = \int_{S}^{S+x} P(t) \, dt = \text{base} \cdot x + \frac{\text{inc}}{2} \left[ (S+x)^2 - S^2 \right]
\]
This function solves the quadratic equation for \( x \) (tokens) given `sol_amount`:
\[
\text{inc} \cdot x^2 + (2 \cdot \text{base} \cdot \text{ONE} + 2 \cdot \text{inc} \cdot S) \cdot x - 2 \cdot \text{ONE}^2 \cdot \text{sol\_amount} = 0
\]
The valid positive root is returned. All arithmetic uses a custom `U256` type for 256‑bit precision.

### `bonding_curve_sell(state, token_amount) -> sol_returned`
Sells `token_amount` tokens, burning them and returning SOL. The SOL amount is computed via `curve_sell_proceeds_readonly` (which uses the integral in reverse):
\[
\text{sol} = \int_{S-x}^{S} P(t) \, dt = \text{base} \cdot x + \frac{\text{inc}}{2} \left[ S^2 - (S-x)^2 \right]
\]

### `curve_sell_proceeds_readonly(state, token_amount) -> sol_returned`
Same as the sell integral but without mutating state – used by the USDT swap to compute the SOL equivalent without actually burning tokens.

### `usd_to_gns(usd_cents, gns_price_cents) -> gns_amount`
Converts a USD amount (in cents) to GNS tokens:
\[
\text{gns} = \frac{\text{usd\_cents} \times \text{ONE\_TOKEN}}{\text{gns\_price\_cents}}
\]

### `gns_raw_to_usd_1e6(raw_amount, gns_usd_1e6) -> usd_1e6`
Converts raw GNS amounts to USD with 6‑decimal precision.

### `compute_pnl(side, margin_gns, leverage, entry_price, current_price, gns_usd_1e6) -> (profit, loss)`
Calculates the profit/loss for a perps position:
- Notional value = `margin_gns × gns_usd_1e6 / ONE_TOKEN × leverage`
- Price movement in basis points: `(current - entry) / entry × 1e6` (or reversed for shorts)
- PnL in USD = notional × price_move / 1e6
- PnL in GNS = PnL_USD × ONE_TOKEN / gns_usd_1e6
Returns a tuple of `(profit_gns, loss_gns)`.

### `compute_market_cap_usd_1e6(curve, sol_usd_1e6) -> market_cap_usd_1e6`
Computes the market capitalisation of the GNS token:
\[
\text{market\_cap} = \frac{\text{total\_supply} \times \text{price\_lamports} \times \text{sol\_usd\_1e6}}{\text{LAMPORTS\_PER\_SOL} \times \text{ONE\_TOKEN}}
\]

### `verify_merkle_proof(leaf, proof, root) -> bool`
Standard Merkle‑tree verification using the Keccak‑256 hash (Solana’s native syscall `sol_keccak_hasher`).  
It sorts the two child hashes before hashing to prevent second‑preimage attacks.

### `u256_sqrt(n: U256) -> U256`
Newton’s method for integer square root, needed because the `construct_uint!` macro does not provide `.sqrt()`.

---

## 🛡 Security

- Every arithmetic operation uses **checked math** (`.checked_mul`, `.checked_add`, `.checked_div`) to catch overflows.
- The `U256` type prevents 128‑bit overflow in bonding‑curve integrals for large supplies.
- All divisions are validated to avoid divide‑by‑zero.
- The `MAX_TRADE_FRACTION_BPS` constant prevents single transactions from consuming too much supply.

---

## 📂 Where to look

- `contracts/genesis-math/src/math.rs` – This file.
- `contracts/genesis-bonding-curve/` – Where `buy` and `sell` call these functions.
- `contracts/genesis-constants/` – Constants like `ONE_TOKEN`, `LAMPORTS_PER_SOL`, `MAX_TRADE_FRACTION_BPS`.
- `contracts/genesis-perps/` – Where `compute_pnl` is used.

*Built by the Blockkette team. Testnet only.*
