use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::errors::ErrorCode;
use crate::events::{GnsPriceUpdated, PriceUpdated};
use crate::state::{GnsUsdOracle, ProtocolAdmin, SolUsdOracle};

#[derive(Accounts)]
pub struct InitializeSolUsdOracle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = SolUsdOracle::ACCOUNT_SIZE, seeds = [b"sol_usd_oracle"], bump)]
    pub sol_usd_oracle: Account<'info, SolUsdOracle>,
    /// CHECK: oracle authority
    pub oracle_authority: AccountInfo<'info>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateSolUsdPrice<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"sol_usd_oracle"], bump = sol_usd_oracle.bump, constraint = sol_usd_oracle.oracle_authority == authority.key())]
    pub sol_usd_oracle: Account<'info, SolUsdOracle>,
}

#[derive(Accounts)]
pub struct InitializeGnsUsdOracle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = GnsUsdOracle::ACCOUNT_SIZE, seeds = [b"gns_usd_oracle"], bump)]
    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,
    /// CHECK: oracle authority
    pub oracle_authority: AccountInfo<'info>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGnsUsdPrice<'info> {
    pub authority: Signer<'info>,
    #[account(mut, seeds = [b"gns_usd_oracle"], bump = gns_usd_oracle.bump, constraint = gns_usd_oracle.oracle_authority == authority.key())]
    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,
}

pub fn initialize_sol_usd_oracle(ctx: Context<InitializeSolUsdOracle>, initial_price_usd_1e6: u64) -> Result<()> {
    require!(initial_price_usd_1e6 > 0, ErrorCode::ZeroAmount);
    let o = &mut ctx.accounts.sol_usd_oracle;
    o.oracle_authority = ctx.accounts.oracle_authority.key();
    o.price_usd_1e6 = initial_price_usd_1e6;
    o.last_updated = Clock::get()?.unix_timestamp;
    o.bump = ctx.bumps.sol_usd_oracle;
    Ok(())
}

pub fn update_sol_usd_price(ctx: Context<UpdateSolUsdPrice>, price_usd_1e6: u64) -> Result<()> {
    require!(price_usd_1e6 > 0, ErrorCode::ZeroAmount);
    let o = &mut ctx.accounts.sol_usd_oracle;
    require_keys_eq!(ctx.accounts.authority.key(), o.oracle_authority, ErrorCode::Unauthorized);
    o.price_usd_1e6 = price_usd_1e6;
    o.last_updated = Clock::get()?.unix_timestamp;
    emit!(PriceUpdated { market: Pubkey::default(), price_usd_1e6, is_sol_usd: true });
    Ok(())
}

pub fn initialize_gns_usd_oracle(ctx: Context<InitializeGnsUsdOracle>, initial_price_usd_1e6: u64) -> Result<()> {
    require!(initial_price_usd_1e6 > 0, ErrorCode::ZeroAmount);
    let o = &mut ctx.accounts.gns_usd_oracle;
    o.oracle_authority = ctx.accounts.oracle_authority.key();
    o.price_usd_1e6 = initial_price_usd_1e6;
    o.last_updated = Clock::get()?.unix_timestamp;
    o.bump = ctx.bumps.gns_usd_oracle;
    Ok(())
}

pub fn update_gns_usd_price(ctx: Context<UpdateGnsUsdPrice>, price_usd_1e6: u64) -> Result<()> {
    require!(price_usd_1e6 > 0, ErrorCode::ZeroAmount);
    let o = &mut ctx.accounts.gns_usd_oracle;
    require_keys_eq!(ctx.accounts.authority.key(), o.oracle_authority, ErrorCode::Unauthorized);
    o.price_usd_1e6 = price_usd_1e6;
    o.last_updated = Clock::get()?.unix_timestamp;
    emit!(GnsPriceUpdated { price_usd_1e6 });
    Ok(())
}
