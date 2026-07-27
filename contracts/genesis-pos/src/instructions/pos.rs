use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::constants::POS_FEE_LAMPORTS;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::math::bonding_curve_buy;
use crate::state::{BondingCurve, FactoryConfig, MerchantPOS};

#[derive(Accounts)]
pub struct ActivatePOS<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(init_if_needed, payer = merchant, space = MerchantPOS::ACCOUNT_SIZE, seeds = [b"merchant_pos", merchant.key().as_ref()], bump)]
    pub pos_account: Account<'info, MerchantPOS>,
    pub business_mint: Account<'info, Mint>,
    #[account(constraint = business_curve.token_mint == business_mint.key())]
    pub business_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub merchant_gns_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessPOSPayment<'info> {
    #[account(mut)]
    pub customer: Signer<'info>,
    #[account(mut)]
    pub merchant: Signer<'info>,
    pub pos_account: Box<Account<'info, MerchantPOS>>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Box<Account<'info, FactoryConfig>>,
    #[account(mut)]
    pub business_curve: Box<Account<'info, BondingCurve>>,
    #[account(mut)]
    pub business_mint: Box<Account<'info, Mint>>,
    #[account(mut, address = business_curve.sol_vault @ ErrorCode::InvalidVault)]
    /// CHECK: business SOL vault
    pub business_sol_vault: AccountInfo<'info>,
    #[account(mut, token::mint = business_mint, token::authority = customer)]
    pub customer_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        seeds = [b"business_mint_authority", merchant.key().as_ref()],
        bump,
        constraint = business_mint.mint_authority == Some(business_mint_authority.key()).into()
    )]
    /// CHECK: business mint authority (validated)
    pub business_mint_authority: AccountInfo<'info>,
    #[account(mut)]
    pub main_curve: Box<Account<'info, BondingCurve>>,
    #[account(mut, address = main_curve.sol_vault @ ErrorCode::InvalidVault)]
    /// CHECK: main SOL vault
    pub main_sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub main_mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [b"mint_authority_v2"],
        bump,
        constraint = main_mint.mint_authority == Some(mint_authority_main.key()).into()
    )]
    /// CHECK: main mint authority (validated)
    pub mint_authority_main: AccountInfo<'info>,
    #[account(mut)]
    pub merchant_gns_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn activate_pos(ctx: Context<ActivatePOS>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    require_keys_eq!(ctx.accounts.merchant.key(), ctx.accounts.business_curve.admin, ErrorCode::Unauthorized);
    let config = &ctx.accounts.config;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.merchant_gns_ata.to_account_info(), to: ctx.accounts.fee_vault.to_account_info(), authority: ctx.accounts.merchant.to_account_info() }), config.pos_activation_cost_gns)?;
    let pos = &mut ctx.accounts.pos_account;
    pos.owner = ctx.accounts.merchant.key();
    pos.business_token_mint = ctx.accounts.business_mint.key();
    pos.activated = true;
    emit!(POSActivated { merchant: pos.owner, business_token_mint: pos.business_token_mint });
    Ok(())
}

pub fn process_pos_payment(ctx: Context<ProcessPOSPayment>, sol_payment: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    require!(sol_payment > POS_FEE_LAMPORTS, ErrorCode::ZeroAmount);
    let pos = &ctx.accounts.pos_account;
    require!(pos.activated, ErrorCode::POSNotActivated);
    require_keys_eq!(pos.owner, ctx.accounts.merchant.key(), ErrorCode::InvalidMerchant);
    require_keys_eq!(pos.business_token_mint, ctx.accounts.business_mint.key(), ErrorCode::InvalidMerchant);
    let merchant_sol = sol_payment.checked_sub(POS_FEE_LAMPORTS).ok_or(ErrorCode::MathOverflow)?;
    let tokens_minted = bonding_curve_buy(&mut ctx.accounts.business_curve, merchant_sol)?;
    invoke(&system_instruction::transfer(&ctx.accounts.customer.key(), &ctx.accounts.business_sol_vault.key(), merchant_sol), &[ctx.accounts.customer.to_account_info(), ctx.accounts.business_sol_vault.to_account_info(), ctx.accounts.system_program.to_account_info()])?;
    let bump_business_auth = ctx.bumps.business_mint_authority;
    let merchant_key = ctx.accounts.merchant.key();
    let seeds: &[&[u8]] = &[b"business_mint_authority", merchant_key.as_ref(), &[bump_business_auth]];
    token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.key(), MintTo { mint: ctx.accounts.business_mint.to_account_info(), to: ctx.accounts.customer_token_account.to_account_info(), authority: ctx.accounts.business_mint_authority.to_account_info() }, &[seeds]), tokens_minted)?;
    let genesis_fee = bonding_curve_buy(&mut ctx.accounts.main_curve, POS_FEE_LAMPORTS)?;
    invoke(&system_instruction::transfer(&ctx.accounts.customer.key(), &ctx.accounts.main_sol_vault.key(), POS_FEE_LAMPORTS), &[ctx.accounts.customer.to_account_info(), ctx.accounts.main_sol_vault.to_account_info(), ctx.accounts.system_program.to_account_info()])?;
    let bump_mint_auth_main = ctx.bumps.mint_authority_main;
    let main_auth_seeds: &[&[u8]] = &[b"mint_authority_v2", &[bump_mint_auth_main]];
    token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.key(), MintTo { mint: ctx.accounts.main_mint.to_account_info(), to: ctx.accounts.merchant_gns_ata.to_account_info(), authority: ctx.accounts.mint_authority_main.to_account_info() }, &[main_auth_seeds]), genesis_fee)?;
    let half = genesis_fee.checked_div(2).ok_or(ErrorCode::MathOverflow)?;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.merchant_gns_ata.to_account_info(), to: ctx.accounts.fee_vault.to_account_info(), authority: ctx.accounts.merchant.to_account_info() }), half)?;
    emit!(POSPayment { customer: ctx.accounts.customer.key(), merchant: ctx.accounts.merchant.key(), business_token_mint: ctx.accounts.business_mint.key(), sol_paid: sol_payment, tokens_received: tokens_minted, fee_sol: POS_FEE_LAMPORTS, genesis_fee, merchant_genesis_refund: half });
    Ok(())
}
