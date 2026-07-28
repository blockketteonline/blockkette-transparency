use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct FactoryConfig {
    pub admin: Pubkey,
    pub gns_mint: Pubkey,
    pub fee_vault: Pubkey,
    pub gns_usd_price_cents: u64,
    pub default_base_price: u64,
    pub default_price_increment: u64,
    pub pos_activation_cost_gns: u64,
    pub ai_subscription_cost_gns: u64,
    pub ai_subscription_period_secs: i64,
    pub tiers: [Tier; 10],
    pub paused: bool,
}
impl FactoryConfig { pub const ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<Self>(); }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Tier { pub usd_cost: u64, pub token_supply: u64 }

#[account]
#[derive(Default)]
pub struct MerchantPOS {
    pub owner: Pubkey,
    pub business_token_mint: Pubkey,
    pub activated: bool,
}
impl MerchantPOS { pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 1; }

#[account]
#[derive(Default)]
pub struct AISubscription { pub owner: Pubkey, pub expiry: i64 }
impl AISubscription { pub const ACCOUNT_SIZE: usize = 8 + 32 + 8; }
