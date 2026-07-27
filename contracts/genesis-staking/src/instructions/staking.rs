use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::events::{Staked, Unstaked};
use crate::state::{FactoryConfig, ProtocolAdmin, StakeRecord, StakingPool};

#[derive(Accounts)]
pub struct InitStakingPool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = StakingPool::ACCOUNT_SIZE, seeds = [b"staking_pool"], bump)]
    pub staking_pool: Account<'info, StakingPool>,
    pub gns_mint: Account<'info, Mint>,
    #[account(init, payer = payer, token::mint = gns_mint, token::authority = staking_pool)]
    pub pool_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StakeGns<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(init_if_needed, payer = owner, space = StakeRecord::ACCOUNT_SIZE, seeds = [b"stake_record", owner.key().as_ref()], bump)]
    pub stake_record: Account<'info, StakeRecord>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeGns<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(mut)]
    pub staking_pool: Account<'info, StakingPool>,
    #[account(mut, constraint = stake_record.owner == owner.key())]
    pub stake_record: Account<'info, StakeRecord>,
    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SweepDonations<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"staking_pool"], bump = staking_pool.bump)]
    pub staking_pool: Account<'info, StakingPool>,
    pub gns_mint: Account<'info, Mint>,
    #[account(mut, token::mint = gns_mint, token::authority = staking_pool)]
    pub pool_ata: Account<'info, TokenAccount>,
    #[account(mut, token::mint = gns_mint)]
    pub treasury_ata: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub token_program: Program<'info, Token>,
}

pub fn init_staking_pool(ctx: Context<InitStakingPool>) -> Result<()> {
    let pool = &mut ctx.accounts.staking_pool;
    pool.gns_mint = ctx.accounts.gns_mint.key();
    pool.pool_ata = ctx.accounts.pool_ata.key();
    pool.total_shares = 0;
    pool.total_staked = 0;
    pool.bump = ctx.bumps.staking_pool;
    Ok(())
}

pub fn stake_gns(ctx: Context<StakeGns>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let pool = &mut ctx.accounts.staking_pool;
    let record = &mut ctx.accounts.stake_record;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.user_ata.to_account_info(), to: ctx.accounts.pool_ata.to_account_info(), authority: ctx.accounts.owner.to_account_info() }), amount)?;
    let shares = if pool.total_shares == 0 { amount } else {
        (amount as u128).checked_mul(pool.total_shares as u128).ok_or(ErrorCode::MathOverflow)?.checked_div(pool.total_staked as u128).ok_or(ErrorCode::MathOverflow)? as u64
    };
    require!(shares > 0, ErrorCode::ZeroAmount);
    pool.total_staked = pool.total_staked.checked_add(amount).ok_or(ErrorCode::MathOverflow)?;
    pool.total_shares = pool.total_shares.checked_add(shares).ok_or(ErrorCode::MathOverflow)?;
    record.owner = ctx.accounts.owner.key();
    record.share_balance = record.share_balance.checked_add(shares).ok_or(ErrorCode::MathOverflow)?;
    emit!(Staked { user: ctx.accounts.owner.key(), shares, gns_amount: amount });
    Ok(())
}

pub fn unstake_gns(ctx: Context<UnstakeGns>, share_amount: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let pool = &mut ctx.accounts.staking_pool;
    let record = &mut ctx.accounts.stake_record;
    require!(record.share_balance >= share_amount, ErrorCode::InsufficientShares);
    let gns_to_return = (share_amount as u128).checked_mul(pool.total_staked as u128).ok_or(ErrorCode::MathOverflow)?.checked_div(pool.total_shares as u128).ok_or(ErrorCode::MathOverflow)? as u64;
    record.share_balance = record.share_balance.checked_sub(share_amount).ok_or(ErrorCode::MathOverflow)?;
    pool.total_shares = pool.total_shares.checked_sub(share_amount).ok_or(ErrorCode::MathOverflow)?;
    pool.total_staked = pool.total_staked.checked_sub(gns_to_return).ok_or(ErrorCode::MathOverflow)?;
    let seeds: &[&[u8]] = &[b"staking_pool", &[pool.bump]];
    token::transfer(CpiContext::new_with_signer(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.pool_ata.to_account_info(), to: ctx.accounts.user_ata.to_account_info(), authority: pool.to_account_info() }, &[seeds]), gns_to_return)?;
    emit!(Unstaked { user: ctx.accounts.owner.key(), shares: share_amount, gns_amount: gns_to_return });
    Ok(())
}

pub fn sweep_donations(ctx: Context<SweepDonations>) -> Result<()> {
    let pool = &ctx.accounts.staking_pool;
    let vault_balance = ctx.accounts.pool_ata.amount;
    let excess = vault_balance.checked_sub(pool.total_staked).ok_or(ErrorCode::MathOverflow)?;
    require!(excess > 0, ErrorCode::ZeroAmount);
    let seeds: &[&[u8]] = &[b"staking_pool", &[pool.bump]];
    token::transfer(CpiContext::new_with_signer(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.pool_ata.to_account_info(), to: ctx.accounts.treasury_ata.to_account_info(), authority: pool.to_account_info() }, &[seeds]), excess)?;
    Ok(())
}
