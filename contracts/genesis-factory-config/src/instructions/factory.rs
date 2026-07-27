use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::errors::ErrorCode;
use crate::state::{FactoryConfig, ProtocolAdmin, Tier};

#[derive(Accounts)]
pub struct InitializeFactoryConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = FactoryConfig::ACCOUNT_SIZE, seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    pub gns_mint: Account<'info, Mint>,
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateFactoryConfig<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"factory_config"], bump, constraint = config.admin == admin.key())]
    pub config: Account<'info, FactoryConfig>,
}

#[derive(Accounts)]
pub struct PauseProtocol<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"factory_config"], bump, constraint = config.admin == admin.key())]
    pub config: Account<'info, FactoryConfig>,
}

#[derive(Accounts)]
pub struct UnpauseProtocol<'info> {
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"factory_config"], bump, constraint = config.admin == admin.key())]
    pub config: Account<'info, FactoryConfig>,
}

pub fn initialize_factory_config(
    ctx: Context<InitializeFactoryConfig>,
    gns_usd_price_cents: u64,
    default_base_price: u64,
    default_price_increment: u64,
    pos_activation_cost_gns: u64,
    ai_subscription_cost_gns: u64,
    ai_subscription_period_secs: i64,
    tiers: [Tier; 10],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.gns_mint = ctx.accounts.gns_mint.key();
    config.fee_vault = ctx.accounts.fee_vault.key();
    config.gns_usd_price_cents = gns_usd_price_cents;
    config.default_base_price = default_base_price;
    config.default_price_increment = default_price_increment;
    config.pos_activation_cost_gns = pos_activation_cost_gns;
    config.ai_subscription_cost_gns = ai_subscription_cost_gns;
    config.ai_subscription_period_secs = ai_subscription_period_secs;
    config.tiers = tiers;
    config.paused = false;
    Ok(())
}

pub fn update_factory_config(
    ctx: Context<UpdateFactoryConfig>,
    gns_usd_price_cents: Option<u64>,
    default_base_price: Option<u64>,
    default_price_increment: Option<u64>,
    pos_activation_cost_gns: Option<u64>,
    ai_subscription_cost_gns: Option<u64>,
    ai_subscription_period_secs: Option<i64>,
    tiers: Option<[Tier; 10]>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    require_keys_eq!(ctx.accounts.admin.key(), config.admin, ErrorCode::Unauthorized);
    if let Some(v) = gns_usd_price_cents { config.gns_usd_price_cents = v; }
    if let Some(v) = default_base_price { config.default_base_price = v; }
    if let Some(v) = default_price_increment { config.default_price_increment = v; }
    if let Some(v) = pos_activation_cost_gns { config.pos_activation_cost_gns = v; }
    if let Some(v) = ai_subscription_cost_gns { config.ai_subscription_cost_gns = v; }
    if let Some(v) = ai_subscription_period_secs { config.ai_subscription_period_secs = v; }
    if let Some(t) = tiers { config.tiers = t; }
    Ok(())
}

pub fn pause_protocol(ctx: Context<PauseProtocol>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.config.admin, ErrorCode::Unauthorized);
    ctx.accounts.config.paused = true;
    Ok(())
}

pub fn unpause_protocol(ctx: Context<UnpauseProtocol>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.config.admin, ErrorCode::Unauthorized);
    ctx.accounts.config.paused = false;
    Ok(())
}
