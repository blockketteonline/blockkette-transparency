use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct AirdropEscrow {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub escrow_token_account: Pubkey,
    pub merkle_root: [u8; 32],
    pub total_amount: u64,
    pub bump: u8,
    pub required_market_cap_usd_1e6: u64,
}
impl AirdropEscrow { pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8; }

#[account]
#[derive(Default)]
pub struct AirdropClaimStatus {
    pub claimed: bool,
}
impl AirdropClaimStatus { pub const ACCOUNT_SIZE: usize = 8 + 1; }

#[account]
#[derive(Default)]
pub struct UsdtSwapConfig {
    pub admin: Pubkey,
    pub usdt_mint: Pubkey,
    pub usdt_vault: Pubkey,
    pub bump: u8,
    pub vault_authority_bump: u8,
}
impl UsdtSwapConfig { pub const ACCOUNT_SIZE: usize = 8 + 32 + 32 + 32 + 1 + 1; }
