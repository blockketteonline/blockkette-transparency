# Genesis Protocol Error Codes

**Complete enumeration of all custom errors returned by the Genesis Anchor program.**  
Each variant is documented with the condition that triggers it, making the codebase auditable and debuggable.

---

## 🧾 Error Reference

| Error Code | Trigger |
|------------|---------|
| `MaxSupplyExceeded` | Minting would push total supply above `max_supply` |
| `InsufficientSol` | Buyer didn't send enough SOL to cover the bonding curve cost |
| `InsufficientSupply` | Trying to burn more tokens than exist in circulation |
| `MathOverflow` | Arithmetic overflow (protected by checked math) |
| `ZeroAmount` | A required amount is zero |
| `Unauthorized` | Signer does not match the required authority |
| `MaxSupplyBelowCurrentSupply` | Proposed new max supply is lower than current total supply |
| `InvalidNewAdmin` | Proposed new admin is the default pubkey or same as current |
| `InvalidAdminConstant` | The hard‑coded initial admin string could not be parsed |
| `ParameterTooLarge` | A config parameter exceeds allowed bounds |
| `VaultRentExemptViolation` | SOL vault would drop below rent‑exempt minimum |
| `SlippageExceeded` | Actual price moved beyond user's slippage tolerance |
| `ParameterTooSmall` | A config parameter is below the minimum allowed |
| `BasePriceTooLow` | Curve base price is below `MIN_BASE_PRICE` |
| `InvalidTier` | Business token tier index is out of range or downgrade attempted |
| `AlreadyInitialized` | An account that should be init-only was already created |
| `SubscriptionAlreadyActive` | Attempt to activate an already‑active subscription |
| `POSNotActivated` | Merchant POS has not been activated |
| `InvalidMerchant` | The merchant does not own the business token being used |
| `MarketInactive` | The perps market is disabled or symbol mismatch |
| `LeverageTooHigh` | Requested leverage exceeds market's max leverage |
| `StalePrice` | Oracle price is older than `ORACLE_MAX_STALENESS_SECS` |
| `InsufficientFreeMargin` | User's free margin is too low for the operation |
| `InsufficientMargin` | Total margin is insufficient |
| `InvalidSide` | Side must be 0 (long) or 1 (short) |
| `LiquidationNotEligible` | Position hasn't crossed the liquidation threshold |
| `InvalidOracleAuthority` | Oracle update signer does not match oracle authority |
| `SelfLiquidation` | Position owner tried to liquidate their own position |
| `PriceSlippage` | Order price deviates too far from oracle price |
| `OracleFutureTimestamp` | Oracle last_updated is in the future (invalid) |
| `PriceDeviationTooLarge` | Current price vs order price exceeds max deviation |
| `DustNotional` | Position notional is below `MIN_NOTIONAL_USD_1E6` |
| `EmergencyPaused` | Protocol is paused (global emergency stop) |
| `PriceNotReady` | Pending price has not yet passed the delay period |
| `StakingPoolInsufficient` | Not enough GNS in staking pool for withdrawal |
| `InsufficientShares` | User's share balance is too low for the unstake amount |
| `OrderNotFound` | Referenced order account does not exist |
| `OrderAlreadyMatched` | Order has already been filled |
| `CounterpartyMismatch` | Counterparty address doesn't match expected |
| `AirdropAlreadyInitialized` | Airdrop escrow was already set up |
| `AirdropNotYetActive` | Market cap hasn't reached the required threshold |
| `InvalidMerkleProof` | Merkle proof failed verification |
| `AlreadyClaimed` | Airdrop already claimed by this wallet |
| `UsdtSwapNotInitialized` | USDT swap config not initialized |
| `UsdtVaultInsufficient` | USDT vault balance too low to cover the swap |
| `InvalidVault` | The SOL vault address doesn't match the expected one |
| `PendingAdminNotSet` | No pending admin nominated |
| `NotPendingAdmin` | Signer does not match the pending admin |
| `TimelockNotElapsed` | Curve param update is still in timelock period |
| `NoPendingCurveUpdate` | No pending curve parameter update exists |
| `PerpPositionStillOpen` | Attempted action on an open position that must be closed first |
| `NotPositionOwner` | Signer does not own the perps position |
| `InsufficientCollateralForLiquidation` | Position has enough collateral to avoid liquidation |
| `PerpPositionAlreadyClosed` | Position has already been closed |
| `InsufficientVaultBalance` | Vault lamports are too low for the requested operation |
| `TradeSizeTooLarge` | Trade exceeds `MAX_TRADE_FRACTION_BPS` of total supply |
| `CooldownNotElapsed` | Not enough slots have passed since the last trade |

---

## 🔍 Why This Matters

These errors are **returned directly to the user** when a transaction fails. They make the protocol:

- **Auditable** – every failure reason is explicit and documented.
- **Secure** – edge cases (slippage, staleness, overflow) are caught before state changes.
- **Developer‑friendly** – front‑end code can map these variants to user‑facing messages.

---

## 📂 Where to look

- `contracts/genesis-errors/src/errors.rs` – The error enum (this file).
- `contracts/genesis-constants/` – Constants referenced by many of these errors.

*Built by the Blockkette team. Testnet only.*
