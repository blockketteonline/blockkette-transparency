use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::errors::ErrorCode;
use crate::events::AISubscriptionPurchased;
use crate::state::{AISubscription, FactoryConfig};

#[derive(Accounts)]
pub struct PurchaseAISubscription<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(init_if_needed, payer = payer, space = AISubscription::ACCOUNT_SIZE, seeds = [b"ai_subscription", payer.key().as_ref()], bump)]
    pub subscription: Account<'info, AISubscription>,
    #[account(mut)]
    pub user_gns_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    /// CHECK: fee vault
    pub fee_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn purchase_ai_subscription(ctx: Context<PurchaseAISubscription>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let config = &ctx.accounts.config;
    token::transfer(CpiContext::new(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.user_gns_ata.to_account_info(), to: ctx.accounts.fee_vault.to_account_info(), authority: ctx.accounts.payer.to_account_info() }), config.ai_subscription_cost_gns)?;
    let sub = &mut ctx.accounts.subscription;
    let clock = Clock::get()?;
    let new_expiry = if sub.expiry > clock.unix_timestamp {
        sub.expiry.checked_add(config.ai_subscription_period_secs).ok_or(ErrorCode::MathOverflow)?
    } else {
        clock.unix_timestamp.checked_add(config.ai_subscription_period_secs).ok_or(ErrorCode::MathOverflow)?
    };
    sub.owner = ctx.accounts.payer.key();
    sub.expiry = new_expiry;
    emit!(AISubscriptionPurchased { user: sub.owner, expiry: new_expiry });
    Ok(())
}
