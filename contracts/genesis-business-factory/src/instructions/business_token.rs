use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::events::*;
use crate::math::usd_to_gns;
use crate::state::{BondingCurve, FactoryConfig};

#[derive(Accounts)]
pub struct CreateBusinessToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(init, payer = payer, space = BondingCurve::ACCOUNT_SIZE, seeds = [b"business_curve", business_mint.key().as_ref()], bump)]
    pub business_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub business_mint: Account<'info, Mint>,
    /// CHECK: business SOL vault
    pub business_sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_gns_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpgradeBusinessToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(mut)]
    pub business_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub user_gns_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateBusinessCurveParams<'info> {
    pub admin: Signer<'info>,
    #[account(mut, constraint = business_curve.admin == admin.key())]
    pub business_curve: Account<'info, BondingCurve>,
}

pub fn create_business_token(ctx: Context<CreateBusinessToken>, tier_index: u8) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let config = &ctx.accounts.config;
    let tier = config.tiers.get(tier_index as usize).ok_or(ErrorCode::InvalidTier)?;
    let gns_cost = usd_to_gns(tier.usd_cost * 100, config.gns_usd_price_cents)?;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.user_gns_ata.to_account_info(), to: ctx.accounts.fee_vault.to_account_info(), authority: ctx.accounts.payer.to_account_info() }), gns_cost)?;
    let state = &mut ctx.accounts.business_curve;
    state.admin = ctx.accounts.payer.key();
    state.pending_admin = Pubkey::default();
    state.token_mint = ctx.accounts.business_mint.key();
    state.sol_vault = ctx.accounts.business_sol_vault.key();
    state.base_price = config.default_base_price;
    state.price_increment = config.default_price_increment;
    state.total_supply = 0;
    state.max_supply = tier.token_supply;
    state.total_sol_raised = 0;
    state.pending_base_price = 0; state.pending_price_increment = 0; state.pending_max_supply = 0; state.pending_timestamp = 0;
    state.last_trade_slot = 0;
    emit!(BusinessTokenCreated { creator: ctx.accounts.payer.key(), token_mint: ctx.accounts.business_mint.key(), max_supply: tier.token_supply, tier_usd_cost: tier.usd_cost });
    Ok(())
}

pub fn upgrade_business_token(ctx: Context<UpgradeBusinessToken>, tier_index: u8) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let config = &ctx.accounts.config;
    let state = &mut ctx.accounts.business_curve;
    require_keys_eq!(ctx.accounts.payer.key(), state.admin, ErrorCode::Unauthorized);
    let current_tier = config.tiers.iter().position(|t| t.token_supply == state.max_supply).ok_or(ErrorCode::InvalidTier)?;
    let new_tier = config.tiers.get(tier_index as usize).ok_or(ErrorCode::InvalidTier)?;
    require!(tier_index as usize > current_tier, ErrorCode::InvalidTier);
    require!(new_tier.token_supply >= state.max_supply, ErrorCode::MaxSupplyBelowCurrentSupply);
    let additional_cost_usd = new_tier.usd_cost.saturating_sub(config.tiers[current_tier].usd_cost);
    let gns_cost = usd_to_gns(additional_cost_usd * 100, config.gns_usd_price_cents)?;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.user_gns_ata.to_account_info(), to: ctx.accounts.fee_vault.to_account_info(), authority: ctx.accounts.payer.to_account_info() }), gns_cost)?;
    state.max_supply = new_tier.token_supply;
    emit!(BusinessTokenUpgraded { owner: ctx.accounts.payer.key(), token_mint: state.token_mint, new_max_supply: new_tier.token_supply, additional_cost_usd });
    Ok(())
}

pub fn update_business_curve_params(ctx: Context<UpdateBusinessCurveParams>, new_base_price: Option<u64>, new_price_increment: Option<u64>, new_max_supply: Option<u64>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.business_curve.admin, ErrorCode::Unauthorized);
    let state = &mut ctx.accounts.business_curve;
    if let Some(bp) = new_base_price { require!(bp >= crate::constants::MIN_BASE_PRICE, ErrorCode::BasePriceTooLow); state.base_price = bp; }
    if let Some(pi) = new_price_increment { require!(pi == 0 || pi >= 2, ErrorCode::ParameterTooSmall); state.price_increment = pi; }
    if let Some(ms) = new_max_supply { require!(ms >= state.total_supply, ErrorCode::MaxSupplyBelowCurrentSupply); state.max_supply = ms; }
    emit!(CurveParamsUpdated { admin: ctx.accounts.admin.key(), base_price: state.base_price, price_increment: state.price_increment, max_supply: state.max_supply });
    Ok(())
}
