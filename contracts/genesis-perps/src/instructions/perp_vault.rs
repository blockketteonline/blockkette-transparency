use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::events::{PerpDepositEvent, PerpWithdrawEvent};
use crate::state::{FactoryConfig, PerpUser, PerpVault};

#[derive(Accounts)]
pub struct InitializePerpVault<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, space = PerpVault::ACCOUNT_SIZE, seeds = [b"perp_vault"], bump)]
    pub vault: Account<'info, PerpVault>,
    pub gns_mint: Account<'info, Mint>,
    #[account(init, payer = payer, token::mint = gns_mint, token::authority = vault)]
    pub vault_ata: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PerpDeposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(mut)]
    pub user_gns_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault: Account<'info, PerpVault>,
    #[account(init_if_needed, payer = payer, space = PerpUser::ACCOUNT_SIZE, seeds = [b"perp_user", payer.key().as_ref()], bump)]
    pub user_position: Account<'info, PerpUser>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PerpWithdraw<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(mut)]
    pub user_position: Account<'info, PerpUser>,
    #[account(mut)]
    pub vault: Account<'info, PerpVault>,
    #[account(mut)]
    pub vault_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_gns_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn initialize_perp_vault(ctx: Context<InitializePerpVault>) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.gns_mint = ctx.accounts.gns_mint.key();
    vault.vault_ata = ctx.accounts.vault_ata.key();
    vault.bump = ctx.bumps.vault;
    Ok(())
}

pub fn perp_deposit(ctx: Context<PerpDeposit>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.user_gns_ata.to_account_info(), to: ctx.accounts.vault_ata.to_account_info(), authority: ctx.accounts.payer.to_account_info() }), amount)?;
    let user = &mut ctx.accounts.user_position;
    user.owner = ctx.accounts.payer.key();
    user.deposited = user.deposited.checked_add(amount).ok_or(ErrorCode::MathOverflow)?;
    user.position_nonce = 0;
    emit!(PerpDepositEvent { user: user.owner, amount });
    Ok(())
}

pub fn perp_withdraw(ctx: Context<PerpWithdraw>, amount: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let user = &mut ctx.accounts.user_position;
    let free = user.deposited.checked_sub(user.locked_margin).ok_or(ErrorCode::MathOverflow)?;
    require!(free >= amount, ErrorCode::InsufficientFreeMargin);
    let vault_bump = ctx.accounts.vault.bump;
    let seeds: &[&[u8]] = &[b"perp_vault", &[vault_bump]];
    token::transfer(CpiContext::new_with_signer(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.vault_ata.to_account_info(), to: ctx.accounts.user_gns_ata.to_account_info(), authority: ctx.accounts.vault.to_account_info() }, &[seeds]), amount)?;
    user.deposited = user.deposited.checked_sub(amount).ok_or(ErrorCode::MathOverflow)?;
    emit!(PerpWithdrawEvent { user: user.owner, amount });
    Ok(())
}
