use anchor_lang::prelude::*;
use std::str::FromStr;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::state::ProtocolAdmin;

#[derive(Accounts)]
pub struct InitializeProtocolAdmin<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = ProtocolAdmin::ACCOUNT_SIZE,
        seeds = [b"protocol_admin"],
        bump,
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct NominateProtocolAdmin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key()
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
}

#[derive(Accounts)]
pub struct AcceptProtocolAdmin<'info> {
    #[account(mut)]
    pub new_admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.pending_admin == new_admin.key()
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
}

#[derive(Accounts)]
pub struct CancelProtocolAdminTransfer<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"protocol_admin"],
        bump = protocol_admin.bump,
        constraint = protocol_admin.current_admin == admin.key()
    )]
    pub protocol_admin: Account<'info, ProtocolAdmin>,
}

pub fn initialize_protocol_admin(ctx: Context<InitializeProtocolAdmin>) -> Result<()> {
    let initial_admin = Pubkey::from_str(INITIAL_ADMIN_PUBKEY_STR).map_err(|_| ErrorCode::InvalidAdminConstant)?;
    require_keys_eq!(ctx.accounts.payer.key(), initial_admin, ErrorCode::Unauthorized);
    let admin = &mut ctx.accounts.protocol_admin;
    admin.current_admin = initial_admin;
    admin.pending_admin = Pubkey::default();
    admin.bump = ctx.bumps.protocol_admin;
    Ok(())
}

pub fn nominate_protocol_admin(ctx: Context<NominateProtocolAdmin>, new_admin: Pubkey) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.protocol_admin.current_admin, ErrorCode::Unauthorized);
    require!(new_admin != Pubkey::default(), ErrorCode::InvalidNewAdmin);
    ctx.accounts.protocol_admin.pending_admin = new_admin;
    Ok(())
}

pub fn accept_protocol_admin(ctx: Context<AcceptProtocolAdmin>) -> Result<()> {
    let admin = &mut ctx.accounts.protocol_admin;
    require!(admin.pending_admin != Pubkey::default(), ErrorCode::PendingAdminNotSet);
    require_keys_eq!(ctx.accounts.new_admin.key(), admin.pending_admin, ErrorCode::NotPendingAdmin);
    admin.current_admin = admin.pending_admin;
    admin.pending_admin = Pubkey::default();
    Ok(())
}

pub fn cancel_protocol_admin_transfer(ctx: Context<CancelProtocolAdminTransfer>) -> Result<()> {
    require_keys_eq!(ctx.accounts.admin.key(), ctx.accounts.protocol_admin.current_admin, ErrorCode::Unauthorized);
    ctx.accounts.protocol_admin.pending_admin = Pubkey::default();
    Ok(())
}
