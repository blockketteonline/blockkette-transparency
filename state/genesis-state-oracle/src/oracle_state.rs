use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct SolUsdOracle {
    pub oracle_authority: Pubkey,
    pub price_usd_1e6: u64,
    pub last_updated: i64,
    pub bump: u8,
}
impl SolUsdOracle { pub const ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<Self>(); }

#[account]
#[derive(Default)]
pub struct GnsUsdOracle {
    pub oracle_authority: Pubkey,
    pub price_usd_1e6: u64,
    pub last_updated: i64,
    pub bump: u8,
}
impl GnsUsdOracle { pub const ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<Self>(); }
