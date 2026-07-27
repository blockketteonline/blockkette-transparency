use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};
use solana_keccak_hasher as keccak;

use crate::constants::ORACLE_MAX_STALENESS_SECS;
use crate::errors::ErrorCode;
use crate::events::{AirdropClaimed, AirdropInitialized};
use crate::math::{compute_market_cap_usd_1e6, verify_merkle_proof};
use crate::state::{AirdropClaimStatus, AirdropEscrow, BondingCurve, FactoryConfig, ProtocolAdmin, SolUsdOracle};

#[derive(Accounts)]
pub struct InitAirdrop<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, constraint = bonding_curve.token_mint == token_mint.key())]
    pub bonding_curve: Account<'info, BondingCurve>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    #[account(seeds = [b"airdrop_escrow"], bump)]
    /// CHECK: PDA that will sign the transfer
    pub escrow_authority: AccountInfo<'info>,
    #[account(init, payer = admin, space = AirdropEscrow::ACCOUNT_SIZE, seeds = [b"airdrop_escrow"], bump)]
    pub escrow: Account<'info, AirdropEscrow>,
    #[account(init, payer = admin, token::mint = token_mint, token::authority = escrow_authority)]
    pub escrow_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"mint_authority_v2"],
        bump,
        constraint = token_mint.mint_authority == Some(mint_authority.key()).into()
    )]
    /// CHECK: Mint authority PDA (validated)
    pub mint_authority: AccountInfo<'info>,
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
pub struct ClaimAirdrop<'info> {
    #[account(mut)]
    pub claimant: Signer<'info>,
    #[account(mut, token::mint = escrow.mint, token::authority = claimant)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = bonding_curve.token_mint == escrow.mint)]
    pub bonding_curve: Box<Account<'info, BondingCurve>>,
    #[account(seeds = [b"airdrop_escrow"], bump = escrow.bump)]
    pub escrow: Box<Account<'info, AirdropEscrow>>,
    #[account(mut, constraint = escrow_token_account.key() == escrow.escrow_token_account)]
    pub escrow_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: PDA authority for escrow
    #[account(seeds = [b"airdrop_escrow"], bump = escrow.bump)]
    pub escrow_authority: AccountInfo<'info>,
    #[account(init_if_needed, payer = claimant, space = AirdropClaimStatus::ACCOUNT_SIZE, seeds = [b"airdrop_claim", claimant.key().as_ref()], bump)]
    pub claim_status: Box<Account<'info, AirdropClaimStatus>>,
    #[account(seeds = [b"sol_usd_oracle"], bump = sol_usd_oracle.bump)]
    pub sol_usd_oracle: Box<Account<'info, SolUsdOracle>>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Box<Account<'info, FactoryConfig>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_airdrop(
    ctx: Context<InitAirdrop>,
    merkle_root: [u8; 32],
    airdrop_amount: u64,
    required_market_cap_usd_1e6: u64,
) -> Result<()> {
    let curve = &mut ctx.accounts.bonding_curve;
    let escrow = &mut ctx.accounts.escrow;
    require!(escrow.authority == Pubkey::default(), ErrorCode::AirdropAlreadyInitialized);
    let bump_mint_auth = ctx.bumps.mint_authority;
    let mint_auth_seeds: &[&[u8]] = &[b"mint_authority_v2", &[bump_mint_auth]];
    let cpi = MintTo { mint: ctx.accounts.token_mint.to_account_info(), to: ctx.accounts.escrow_token_account.to_account_info(), authority: ctx.accounts.mint_authority.to_account_info() };
    token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.key(), cpi, &[mint_auth_seeds]), airdrop_amount)?;
    curve.total_supply = curve.total_supply.checked_add(airdrop_amount).ok_or(ErrorCode::MathOverflow)?;
    require!(curve.total_supply <= curve.max_supply, ErrorCode::MaxSupplyExceeded);
    escrow.authority = ctx.accounts.escrow_authority.key();
    escrow.mint = curve.token_mint;
    escrow.escrow_token_account = ctx.accounts.escrow_token_account.key();
    escrow.merkle_root = merkle_root;
    escrow.total_amount = airdrop_amount;
    escrow.bump = ctx.bumps.escrow;
    escrow.required_market_cap_usd_1e6 = required_market_cap_usd_1e6;
    emit!(AirdropInitialized { amount: airdrop_amount, merkle_root });
    Ok(())
}

pub fn claim_airdrop(ctx: Context<ClaimAirdrop>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let sol_oracle = &ctx.accounts.sol_usd_oracle;
    let clock = Clock::get()?;
    require!(sol_oracle.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - sol_oracle.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);
    let sol_usd_1e6 = sol_oracle.price_usd_1e6;
    let market_cap = compute_market_cap_usd_1e6(&ctx.accounts.bonding_curve, sol_usd_1e6)?;
    let required_cap = ctx.accounts.escrow.required_market_cap_usd_1e6 as u128;
    if required_cap > 0 { require!(market_cap >= required_cap, ErrorCode::AirdropNotYetActive); }
    let claimant = ctx.accounts.claimant.key();
    let mut leaf_data = Vec::new();
    leaf_data.extend_from_slice(&claimant.to_bytes());
    leaf_data.extend_from_slice(&amount.to_le_bytes());
    let leaf_hash = keccak::hashv(&[&leaf_data]);
    require!(verify_merkle_proof(leaf_hash.to_bytes(), proof, ctx.accounts.escrow.merkle_root), ErrorCode::InvalidMerkleProof);
    let claim_status = &mut ctx.accounts.claim_status;
    require!(!claim_status.claimed, ErrorCode::AlreadyClaimed);
    claim_status.claimed = true;
    let escrow_bump = ctx.accounts.escrow.bump;
    let seeds: &[&[u8]] = &[b"airdrop_escrow", &[escrow_bump]];
    token::transfer(CpiContext::new_with_signer(ctx.accounts.token_program.key(), Transfer { from: ctx.accounts.escrow_token_account.to_account_info(), to: ctx.accounts.user_token_account.to_account_info(), authority: ctx.accounts.escrow_authority.to_account_info() }, &[seeds]), amount)?;
    emit!(AirdropClaimed { claimant, amount });
    Ok(())
}
