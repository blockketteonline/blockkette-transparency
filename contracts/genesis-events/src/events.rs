use anchor_lang::prelude::*;

#[event] pub struct CurveInitialized { pub admin: Pubkey, pub token_mint: Pubkey, pub sol_vault: Pubkey, pub base_price: u64, pub price_increment: u64, pub max_supply: u64 }
#[event] pub struct AdminUpdated { pub old_admin: Pubkey, pub new_admin: Pubkey }
#[event] pub struct CurveParamsUpdated { pub admin: Pubkey, pub base_price: u64, pub price_increment: u64, pub max_supply: u64 }
#[event] pub struct BuyEvent { pub buyer: Pubkey, pub sol_amount: u64, pub tokens_minted: u64, pub new_total_supply: u64, pub current_price: u64 }
#[event] pub struct SellEvent { pub seller: Pubkey, pub tokens_burned: u64, pub sol_returned: u64, pub new_total_supply: u64, pub current_price: u64 }
#[event] pub struct BusinessTokenCreated { pub creator: Pubkey, pub token_mint: Pubkey, pub max_supply: u64, pub tier_usd_cost: u64 }
#[event] pub struct BusinessTokenUpgraded { pub owner: Pubkey, pub token_mint: Pubkey, pub new_max_supply: u64, pub additional_cost_usd: u64 }
#[event] pub struct POSActivated { pub merchant: Pubkey, pub business_token_mint: Pubkey }
#[event] pub struct POSPayment { pub customer: Pubkey, pub merchant: Pubkey, pub business_token_mint: Pubkey, pub sol_paid: u64, pub tokens_received: u64, pub fee_sol: u64, pub genesis_fee: u64, pub merchant_genesis_refund: u64 }
#[event] pub struct AISubscriptionPurchased { pub user: Pubkey, pub expiry: i64 }
#[event] pub struct PerpDepositEvent { pub user: Pubkey, pub amount: u64 }
#[event] pub struct PerpWithdrawEvent { pub user: Pubkey, pub amount: u64 }
#[event] pub struct PerpMarketCreated { pub market: Pubkey, pub symbol: [u8; 16], pub max_leverage: u16, pub taker_fee_bps: u16, pub max_deviation_bps: u16, pub price_delay_secs: i64 }
#[event] pub struct PriceUpdated { pub market: Pubkey, pub price_usd_1e6: u64, pub is_sol_usd: bool }
#[event] pub struct GnsPriceUpdated { pub price_usd_1e6: u64 }
#[event] pub struct OrderPlaced { pub owner: Pubkey, pub side: u8, pub margin_gns: u64, pub leverage: u16, pub price_usd_1e6: u64 }
#[event] pub struct OrderMatched { pub maker: Pubkey, pub taker: Pubkey, pub price_usd_1e6: u64, pub margin_gns: u64, pub leverage: u16 }
#[event] pub struct PositionOpened { pub owner: Pubkey, pub market: Pubkey, pub side: u8, pub margin_gns: u64, pub leverage: u16, pub entry_price_usd_1e6: u64 }
#[event] pub struct PositionClosed { pub owner: Pubkey, pub market: Pubkey, pub payout_gns: u64, pub margin_gns: u64, pub profit_gns: u64, pub loss_gns: u64 }
#[event] pub struct PositionLiquidated { pub owner: Pubkey, pub market: Pubkey, pub liquidator: Pubkey, pub payout_gns: u64 }
#[event] pub struct Staked { pub user: Pubkey, pub shares: u64, pub gns_amount: u64 }
#[event] pub struct Unstaked { pub user: Pubkey, pub shares: u64, pub gns_amount: u64 }
#[event] pub struct AirdropInitialized { pub amount: u64, pub merkle_root: [u8; 32] }
#[event] pub struct AirdropClaimed { pub claimant: Pubkey, pub amount: u64 }
#[event] pub struct GnsForUsdtSwap { pub user: Pubkey, pub gns_burned: u64, pub usdt_received: u64, pub sol_equivalent: u64 }
