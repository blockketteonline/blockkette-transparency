use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::math::{bonding_curve_buy, bonding_curve_sell, get_current_price};
use crate::state::{BondingCurve, FactoryConfig, ProtocolAdmin};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = BondingCurve::ACCOUNT_SIZE)]
    pub state: Account<'info, BondingCurve>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    /// CHECK: SOL vault PDA – validated by rent‑exemption constraint
    #[account(
        mut,
        constraint = sol_vault.lamports() >= VAULT_RENT_EXEMPT_MIN @ ErrorCode::VaultRentExemptViolation
    )]
    pub sol_vault: AccountInfo<'info>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut, constraint = state.token_mint == token_mint.key())]
    pub state: Account<'info, BondingCurve>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = token_mint, token::authority = buyer)]
    pub buyer_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = state.sol_vault @ ErrorCode::InvalidVault)]
    /// CHECK: SOL vault
    pub sol_vault: AccountInfo<'info>,
    #[account(
        seeds = [b"mint_authority_v2"],
        bump,
        constraint = token_mint.mint_authority == Some(mint_authority.key()).into()
    )]
    /// CHECK: Mint authority PDA (validated)
    pub mint_authority: AccountInfo<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Sell<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut, constraint = state.token_mint == token_mint.key())]
    pub state: Account<'info, BondingCurve>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = token_mint, token::authority = seller)]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = state.sol_vault @ ErrorCode::InvalidVault)]
    /// CHECK: SOL vault
    pub sol_vault: AccountInfo<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct NominateAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, constraint = state.admin == admin.key())]
    pub state: Account<'info, BondingCurve>,
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    #[account(mut)]
    pub new_admin: Signer<'info>,
    #[account(mut, constraint = state.pending_admin == new_admin.key())]
    pub state: Account<'info, BondingCurve>,
}

#[derive(Accounts)]
pub struct CancelAdminTransfer<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, constraint = state.admin == admin.key())]
    pub state: Account<'info, BondingCurve>,
}

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, BondingCurve>,
}

pub fn initialize(ctx: Context<Initialize>, base_price: u64, price_inc: u64, max_supply: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.admin = ctx.accounts.admin.key();
    state.pending_admin = Pubkey::default();
    state.token_mint = ctx.accounts.token_mint.key();
    state.sol_vault = ctx.accounts.sol_vault.key();
    state.base_price = base_price;
    state.price_increment = price_inc;
    state.total_supply = 0;
    state.max_supply = max_supply;
    state.total_sol_raised = 0;
    state.pending_base_price = 0;
    state.pending_price_increment = 0;
    state.pending_max_supply = 0;
    state.pending_timestamp = 0;
    state.last_trade_slot = 0;

    emit!(CurveInitialized { admin: state.admin, token_mint: state.token_mint, sol_vault: state.sol_vault, base_price, price_increment: price_inc, max_supply });
    Ok(())
}

pub fn buy(ctx: Context<Buy>, amount: u64, min_tokens_out: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);

    let state = &mut ctx.accounts.state;
    let clock = Clock::get()?;
    require!(clock.slot > state.last_trade_slot, ErrorCode::CooldownNotElapsed);
    state.last_trade_slot = clock.slot;

    let tokens = bonding_curve_buy(state, amount)?;
    require!(tokens >= min_tokens_out, ErrorCode::SlippageExceeded);

    let ix = system_instruction::transfer(&ctx.accounts.buyer.key(), &ctx.accounts.sol_vault.key(), amount);
    invoke(&ix, &[ctx.accounts.buyer.to_account_info(), ctx.accounts.sol_vault.to_account_info(), ctx.accounts.system_program.to_account_info()])?;

    let bump_mint_auth = ctx.bumps.mint_authority;
    let mint_auth_seeds: &[&[u8]] = &[b"mint_authority_v2", &[bump_mint_auth]];
    let cpi = MintTo { mint: ctx.accounts.token_mint.to_account_info(), to: ctx.accounts.buyer_token_account.to_account_info(), authority: ctx.accounts.mint_authority.to_account_info() };
    token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi, &[mint_auth_seeds]), tokens)?;

    let price = get_current_price(&ctx.accounts.state)?;
    emit!(BuyEvent { buyer: ctx.accounts.buyer.key(), sol_amount: amount, tokens_minted: tokens, new_total_supply: ctx.accounts.state.total_supply, current_price: price });
    Ok(())
}

pub fn sell(ctx: Context<Sell>, amount: u64, min_sol_out: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);

    let state = &mut ctx.accounts.state;
    let clock = Clock::get()?;
    require!(clock.slot > state.last_trade_slot, ErrorCode::CooldownNotElapsed);
    state.last_trade_slot = clock.slot;

    let sol = bonding_curve_sell(state, amount)?;
    require!(sol >= min_sol_out, ErrorCode::SlippageExceeded);

    token::burn(CpiContext::new(ctx.accounts.token_program.key(), Burn { mint: ctx.accounts.token_mint.to_account_info(), from: ctx.accounts.seller_token_account.to_account_info(), authority: ctx.accounts.seller.to_account_info() }), amount)?;

    let vault = ctx.accounts.sol_vault.to_account_info();
    let seller = ctx.accounts.seller.to_account_info();
    let remaining = vault.lamports().checked_sub(sol).ok_or(ErrorCode::InsufficientVaultBalance)?;
    require!(remaining >= VAULT_RENT_EXEMPT_MIN, ErrorCode::VaultRentExemptViolation);
    **vault.try_borrow_mut_lamports()? = remaining;
    **seller.try_borrow_mut_lamports()? = seller.lamports().checked_add(sol).ok_or(ErrorCode::MathOverflow)?;

    let price = get_current_price(&ctx.accounts.state)?;
    emit!(SellEvent { seller: ctx.accounts.seller.key(), tokens_burned: amount, sol_returned: sol, new_total_supply: ctx.accounts.state.total_supply, current_price: price });
    Ok(())
}

pub fn nominate_admin(ctx: Context<NominateAdmin>, new_admin: Pubkey) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
    require!(new_admin != Pubkey::default(), ErrorCode::InvalidNewAdmin);
    require!(new_admin != state.admin, ErrorCode::InvalidNewAdmin);
    state.pending_admin = new_admin;
    Ok(())
}

pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require!(state.pending_admin != Pubkey::default(), ErrorCode::PendingAdminNotSet);
    require_keys_eq!(ctx.accounts.new_admin.key(), state.pending_admin, ErrorCode::NotPendingAdmin);
    let old_admin = state.admin;
    state.admin = state.pending_admin;
    state.pending_admin = Pubkey::default();
    emit!(AdminUpdated { old_admin, new_admin: state.admin });
    Ok(())
}

pub fn cancel_admin_transfer(ctx: Context<CancelAdminTransfer>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require_keys_eq!(ctx.accounts.admin.key(), state.admin, ErrorCode::Unauthorized);
    state.pending_admin = Pubkey::default();
    Ok(())
}

pub fn update_curve_params(ctx: Context<UpdateAdmin>, new_base_price: Option<u64>, new_price_increment: Option<u64>, new_max_supply: Option<u64>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.state.admin, ErrorCode::Unauthorized);
    let state = &mut ctx.accounts.state;
    let clock = Clock::get()?;
    let bp = new_base_price.unwrap_or(state.base_price);
    let pi = new_price_increment.unwrap_or(state.price_increment);
    let ms = new_max_supply.unwrap_or(state.max_supply);
    require!(bp >= MIN_BASE_PRICE, ErrorCode::BasePriceTooLow);
    require!(pi == 0 || pi >= 2, ErrorCode::ParameterTooSmall);
    require!(ms >= state.total_supply, ErrorCode::MaxSupplyBelowCurrentSupply);
    state.pending_base_price = bp;
    state.pending_price_increment = pi;
    state.pending_max_supply = ms;
    state.pending_timestamp = clock.unix_timestamp + CURVE_PARAM_TIMELOCK_SECS;
    Ok(())
}

pub fn execute_curve_params_update(ctx: Context<UpdateAdmin>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require!(state.pending_timestamp > 0, ErrorCode::NoPendingCurveUpdate);
    let clock = Clock::get()?;
    require!(clock.unix_timestamp >= state.pending_timestamp, ErrorCode::TimelockNotElapsed);
    state.base_price = state.pending_base_price;
    state.price_increment = state.pending_price_increment;
    state.max_supply = state.pending_max_supply;
    state.pending_base_price = 0;
    state.pending_price_increment = 0;
    state.pending_max_supply = 0;
    state.pending_timestamp = 0;
    emit!(CurveParamsUpdated { admin: state.admin, base_price: state.base_price, price_increment: state.price_increment, max_supply: state.max_supply });
    Ok(())
}

pub fn cancel_curve_params_update(ctx: Context<UpdateAdmin>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.state.admin, ErrorCode::Unauthorized);
    let state = &mut ctx.accounts.state;
    state.pending_base_price = 0;
    state.pending_price_increment = 0;
    state.pending_max_supply = 0;
    state.pending_timestamp = 0;
    Ok(())
}
