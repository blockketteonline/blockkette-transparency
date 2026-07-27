use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

use crate::constants::{LAMPORTS_PER_SOL, ORACLE_MAX_STALENESS_SECS, VAULT_RENT_EXEMPT_MIN};
use crate::errors::ErrorCode;
use crate::events::GnsForUsdtSwap;
use crate::math::curve_sell_proceeds_readonly;
use crate::state::{BondingCurve, FactoryConfig, ProtocolAdmin, SolUsdOracle, UsdtSwapConfig};

#[derive(Accounts)]
pub struct InitUsdtSwapConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = UsdtSwapConfig::ACCOUNT_SIZE, seeds = [b"usdt_swap_config"], bump)]
    pub usdt_swap_config: Account<'info, UsdtSwapConfig>,
    /// CHECK: PDA authority for USDT vault
    #[account(seeds = [b"usdt_swap_auth"], bump)]
    pub usdt_vault_authority: AccountInfo<'info>,
    #[account(init, payer = admin, token::mint = usdt_mint, token::authority = usdt_vault_authority)]
    pub usdt_vault_token_account: Account<'info, TokenAccount>,
    pub usdt_mint: Account<'info, Mint>,
    #[account(
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key() @ ErrorCode::Unauthorized
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SwapGnsForUsdt<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, token::mint = gns_mint, token::authority = user)]
    pub user_gns_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = usdt_mint, token::authority = user)]
    pub user_usdt_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = bonding_curve.token_mint == gns_mint.key())]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    pub gns_mint: Box<Account<'info, Mint>>,
    #[account(
        constraint = usdt_swap_config.usdt_mint == usdt_mint.key()
    )]
    pub usdt_mint: Box<Account<'info, Mint>>,
    #[account(
        seeds = [b"usdt_swap_config"],
        bump = usdt_swap_config.bump,
    )]
    pub usdt_swap_config: Box<Account<'info, UsdtSwapConfig>>,
    /// CHECK: PDA authority for the USDT vault
    #[account(
        seeds = [b"usdt_swap_auth"],
        bump = usdt_swap_config.vault_authority_bump,
    )]
    pub usdt_vault_authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = usdt_vault_token_account.key() == usdt_swap_config.usdt_vault,
        token::mint = usdt_mint,
    )]
    pub usdt_vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        seeds = [b"sol_usd_oracle"],
        bump = sol_usd_oracle.bump,
    )]
    pub sol_usd_oracle: Box<Account<'info, SolUsdOracle>>,
    #[account(
        seeds = [b"factory_config"],
        bump
    )]
    pub config: Box<Account<'info, FactoryConfig>>,

    /// CHECK: SOL vault of the bonding curve
    #[account(
        mut,
        address = bonding_curve.sol_vault @ ErrorCode::InvalidVault
    )]
    pub sol_vault: AccountInfo<'info>,

    /// CHECK: SOL treasury PDA that receives the SOL from the simulated sell
    #[account(
        mut,
        seeds = [b"sol_treasury"],
        bump
    )]
    pub sol_treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn init_usdt_swap_config(ctx: Context<InitUsdtSwapConfig>, usdt_mint: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.usdt_swap_config;
    config.admin = ctx.accounts.admin.key();
    config.usdt_mint = usdt_mint;
    config.usdt_vault = ctx.accounts.usdt_vault_token_account.key();
    config.bump = ctx.bumps.usdt_swap_config;
    config.vault_authority_bump = ctx.bumps.usdt_vault_authority;
    Ok(())
}

pub fn swap_gns_for_usdt(ctx: Context<SwapGnsForUsdt>, gns_amount: u64, min_usdt_out: u64) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    require!(gns_amount > 0, ErrorCode::ZeroAmount);

    let curve = &ctx.accounts.bonding_curve;
    let sol_proceeds = curve_sell_proceeds_readonly(curve, gns_amount)?;

    let sol_oracle = &ctx.accounts.sol_usd_oracle;
    let clock = Clock::get()?;
    require!(sol_oracle.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - sol_oracle.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);
    let sol_usd_1e6 = sol_oracle.price_usd_1e6;

    let usdt_mint = &ctx.accounts.usdt_mint;
    let usdt_decimals = usdt_mint.decimals;
    let usdt_factor = 10u64.pow(usdt_decimals as u32);
    let usdt_out = (sol_proceeds as u128)
        .checked_mul(sol_usd_1e6 as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(LAMPORTS_PER_SOL as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_mul(usdt_factor as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(1_000_000u128).ok_or(ErrorCode::MathOverflow)?;
    let usdt_out = u64::try_from(usdt_out).map_err(|_| ErrorCode::MathOverflow)?;
    require!(usdt_out >= min_usdt_out, ErrorCode::SlippageExceeded);

    let vault_balance = ctx.accounts.usdt_vault_token_account.amount;
    require!(vault_balance >= usdt_out, ErrorCode::UsdtVaultInsufficient);

    token::burn(CpiContext::new(
        ctx.accounts.token_program.key(),
        Burn {
            mint: ctx.accounts.gns_mint.to_account_info(),
            from: ctx.accounts.user_gns_ata.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        },
    ), gns_amount)?;

    let curve = &mut ctx.accounts.bonding_curve;
    curve.total_supply = curve.total_supply.checked_sub(gns_amount).ok_or(ErrorCode::MathOverflow)?;
    curve.total_sol_raised = curve.total_sol_raised.checked_sub(sol_proceeds).ok_or(ErrorCode::MathOverflow)?;

    {
        let vault = &ctx.accounts.sol_vault;
        let treasury = &ctx.accounts.sol_treasury;
        let vault_lamports = vault.lamports();
        let remaining = vault_lamports.checked_sub(sol_proceeds).ok_or(ErrorCode::InsufficientVaultBalance)?;
        require!(remaining >= VAULT_RENT_EXEMPT_MIN, ErrorCode::VaultRentExemptViolation);

        **vault.try_borrow_mut_lamports()? = remaining;
        **treasury.try_borrow_mut_lamports()? = treasury.lamports()
            .checked_add(sol_proceeds)
            .ok_or(ErrorCode::MathOverflow)?;
    }

    let config = &ctx.accounts.usdt_swap_config;
    let vault_bump = config.vault_authority_bump;
    let seeds: &[&[u8]] = &[b"usdt_swap_auth", &[vault_bump]];
    token::transfer(CpiContext::new_with_signer(
        ctx.accounts.token_program.key(),
        Transfer {
            from: ctx.accounts.usdt_vault_token_account.to_account_info(),
            to: ctx.accounts.user_usdt_ata.to_account_info(),
            authority: ctx.accounts.usdt_vault_authority.to_account_info(),
        },
        &[seeds],
    ), usdt_out)?;

    emit!(GnsForUsdtSwap {
        user: ctx.accounts.user.key(),
        gns_burned: gns_amount,
        usdt_received: usdt_out,
        sol_equivalent: sol_proceeds,
    });
    Ok(())
}
