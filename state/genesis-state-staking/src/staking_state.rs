use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct StakingPool {
    pub gns_mint: Pubkey,
    pub pool_ata: Pubkey,
    pub total_shares: u64,
    pub total_staked: u64,
    pub bump: u8,
}
impl StakingPool { pub const ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<Self>(); }

#[account]
#[derive(Default)]
pub struct StakeRecord {
    pub owner: Pubkey,
    pub share_balance: u64,
}
impl StakeRecord { pub const ACCOUNT_SIZE: usize = 8 + std::mem::size_of::<Self>(); }
