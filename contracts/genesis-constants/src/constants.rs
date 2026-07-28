// This key is only used once to initialise the ProtocolAdmin PDA.
pub const INITIAL_ADMIN_PUBKEY_STR: &str = "FCjRhDx4BtuTR86rmdgTw5cSAXJejbt7CvmGUP7rwWy2";
pub const DECIMALS: u32 = 9;
pub const ONE_TOKEN: u64 = 10u64.pow(DECIMALS);
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const MIN_BASE_PRICE: u64 = 10_000;
pub const VAULT_RENT_EXEMPT_MIN: u64 = 890_880;

pub const POS_FEE_LAMPORTS: u64 = 10_000;

pub const ORACLE_MAX_STALENESS_SECS: i64 = 90;
pub const MAINTENANCE_MARGIN_BPS: u64 = 1_000;
pub const LIQUIDATION_BOUNTY_GNS: u64 = 10 * ONE_TOKEN;

pub const MIN_NOTIONAL_USD_1E6: u128 = 10_000;

pub const TREASURY_FEE_BPS: u64 = 5_000;
pub const STAKING_REWARD_BPS: u64 = 5_000;

pub const CLOSE_INITIATOR_FEE_GNS: u64 = 1_000_000;

pub const CURVE_PARAM_TIMELOCK_SECS: i64 = 86400;

pub const MAX_TRADE_FRACTION_BPS: u64 = 100;
