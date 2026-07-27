use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::math::{compute_pnl, gns_raw_to_usd_1e6};
use crate::state::{FactoryConfig, GnsUsdOracle, PerpMarket, PerpOrder, PerpPosition, PerpUser};

#[derive(Accounts)]
#[instruction(symbol: [u8; 16])]
pub struct CreatePerpMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(init, payer = admin, space = PerpMarket::ACCOUNT_SIZE, seeds = [b"perp_market", symbol.as_ref()], bump)]
    pub market: Account<'info, PerpMarket>,
    /// CHECK: oracle authority
    pub oracle_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(symbol: [u8; 16], side: u8, margin_gns: u64, leverage: u16, price_usd_1e6: u64)]
pub struct CreatePerpOrder<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,
    #[account(seeds = [b"perp_market", &symbol], bump = market.bump, constraint = market.active)]
    pub market: Account<'info, PerpMarket>,
    #[account(seeds = [b"gns_usd_oracle"], bump = gns_usd_oracle.bump)]
    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,
    #[account(init, payer = owner, space = PerpOrder::ACCOUNT_SIZE, seeds = [b"perp_order", owner.key().as_ref(), &market.key().as_ref(), &[side], &margin_gns.to_le_bytes(), &leverage.to_le_bytes(), &price_usd_1e6.to_le_bytes()], bump)]
    pub order: Account<'info, PerpOrder>,
    #[account(mut, seeds = [b"perp_user", owner.key().as_ref()], bump, constraint = perp_user.owner == owner.key())]
    pub perp_user: Account<'info, PerpUser>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TakePerpOrder<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,

    #[account(mut, constraint = order.market == market.key())]
    pub order: Account<'info, PerpOrder>,

    #[account(mut)]
    pub market: Account<'info, PerpMarket>,

    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,

    #[account(mut, constraint = maker_perp_user.owner == order.owner)]
    pub maker_perp_user: Account<'info, PerpUser>,

    #[account(mut, constraint = taker_perp_user.owner == taker.key())]
    pub taker_perp_user: Account<'info, PerpUser>,

    #[account(
        init,
        payer = taker,
        space = PerpPosition::ACCOUNT_SIZE,
        seeds = [b"perp_position", order.owner.as_ref(), market.key().as_ref(), &maker_perp_user.position_nonce.to_le_bytes()],
        bump,
    )]
    pub maker_position: Account<'info, PerpPosition>,

    #[account(
        init,
        payer = taker,
        space = PerpPosition::ACCOUNT_SIZE,
        seeds = [b"perp_position", taker.key().as_ref(), market.key().as_ref(), &taker_perp_user.position_nonce.to_le_bytes()],
        bump,
    )]
    pub taker_position: Account<'info, PerpPosition>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePerpPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,

    #[account(
        mut,
        close = owner,
        seeds = [b"perp_position", position.owner.as_ref(), position.market.as_ref(), &position.nonce.to_le_bytes()],
        bump = position.bump,
        constraint = position.owner == owner.key() @ ErrorCode::NotPositionOwner,
        constraint = position.margin_gns > 0
    )]
    pub position: Account<'info, PerpPosition>,

    #[account(mut, constraint = user.owner == owner.key())]
    pub user: Account<'info, PerpUser>,

    #[account(mut)]
    pub market: Account<'info, PerpMarket>,
    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,
    #[account(seeds = [b"factory_config"], bump)]
    pub config: Account<'info, FactoryConfig>,

    #[account(
        mut,
        close = position_owner_wallet,
        seeds = [b"perp_position", position.owner.as_ref(), position.market.as_ref(), &position.nonce.to_le_bytes()],
        bump = position.bump,
        constraint = position.margin_gns > 0
    )]
    pub position: Account<'info, PerpPosition>,

    /// CHECK: wallet that receives the rent (must be the position owner)
    #[account(mut, constraint = position_owner_wallet.key() == position.owner)]
    pub position_owner_wallet: AccountInfo<'info>,

    #[account(mut, constraint = user.owner == position.owner)]
    pub user: Account<'info, PerpUser>,

    #[account(mut, constraint = liquidator_user.owner == liquidator.key())]
    pub liquidator_user: Account<'info, PerpUser>,

    #[account(mut)]
    pub market: Account<'info, PerpMarket>,
    pub gns_usd_oracle: Account<'info, GnsUsdOracle>,

    pub system_program: Program<'info, System>,
}

pub fn create_perp_market(
    ctx: Context<CreatePerpMarket>,
    symbol: [u8; 16],
    max_leverage: u16,
    taker_fee_bps: u16,
    initial_price_usd_1e6: u64,
    max_deviation_bps: u16,
    price_delay_secs: i64,
) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    require!(max_leverage >= 1 && max_leverage <= 100, ErrorCode::LeverageTooHigh);
    require!(initial_price_usd_1e6 > 0, ErrorCode::ZeroAmount);
    let m = &mut ctx.accounts.market;
    m.admin = ctx.accounts.admin.key();
    m.symbol = symbol;
    m.oracle_authority = ctx.accounts.oracle_authority.key();
    m.price_usd_1e6 = initial_price_usd_1e6;
    m.last_updated = Clock::get()?.unix_timestamp;
    m.max_leverage = max_leverage;
    m.taker_fee_bps = taker_fee_bps;
    m.max_deviation_bps = max_deviation_bps;
    m.active = true;
    m.open_interest_long_gns = 0;
    m.open_interest_short_gns = 0;
    m.bump = ctx.bumps.market;
    m.price_delay_secs = price_delay_secs;
    m.pending_price_usd_1e6 = initial_price_usd_1e6;
    m.pending_price_ts = m.last_updated;
    emit!(PerpMarketCreated { market: m.key(), symbol, max_leverage, taker_fee_bps, max_deviation_bps, price_delay_secs });
    Ok(())
}

pub fn create_perp_order(
    ctx: Context<CreatePerpOrder>,
    symbol: [u8; 16],
    side: u8,
    margin_gns: u64,
    leverage: u16,
    price_usd_1e6: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let market = &ctx.accounts.market;
    require!(market.active, ErrorCode::MarketInactive);
    require!(market.symbol == symbol, ErrorCode::MarketInactive);
    require!(leverage <= market.max_leverage, ErrorCode::LeverageTooHigh);
    require!(side == 0 || side == 1, ErrorCode::InvalidSide);
    require!(margin_gns > 0, ErrorCode::ZeroAmount);

    let clock = Clock::get()?;
    require!(market.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - market.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);

    let gns_oracle = &ctx.accounts.gns_usd_oracle;
    let notional = gns_raw_to_usd_1e6(margin_gns, gns_oracle.price_usd_1e6)?
        .checked_mul(leverage as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    require!(notional >= MIN_NOTIONAL_USD_1E6, ErrorCode::DustNotional);

    let user = &mut ctx.accounts.perp_user;
    require!(user.owner == ctx.accounts.owner.key(), ErrorCode::Unauthorized);
    let free_margin = user.deposited.checked_sub(user.locked_margin).ok_or(ErrorCode::MathOverflow)?;
    require!(free_margin >= margin_gns, ErrorCode::InsufficientFreeMargin);

    user.locked_margin = user.locked_margin.checked_add(margin_gns).ok_or(ErrorCode::MathOverflow)?;

    let order = &mut ctx.accounts.order;
    order.owner = ctx.accounts.owner.key();
    order.market = market.key();
    order.side = side;
    order.margin_gns = margin_gns;
    order.leverage = leverage;
    order.price_usd_1e6 = price_usd_1e6;
    order.bump = ctx.bumps.order;
    order.matched = false;

    emit!(OrderPlaced { owner: order.owner, side, margin_gns, leverage, price_usd_1e6 });
    Ok(())
}

pub fn take_perp_order(ctx: Context<TakePerpOrder>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    require!(!ctx.accounts.order.matched, ErrorCode::OrderAlreadyMatched);
    require!(ctx.accounts.market.active, ErrorCode::MarketInactive);
    require_keys_eq!(ctx.accounts.order.market, ctx.accounts.market.key(), ErrorCode::MarketInactive);

    let clock = Clock::get()?;
    require!(ctx.accounts.market.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - ctx.accounts.market.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);

    let current_price = ctx.accounts.market.price_usd_1e6;
    let order_price = ctx.accounts.order.price_usd_1e6;
    let price_diff = if current_price > order_price { current_price - order_price } else { order_price - current_price };
    let max_deviation_bps = ctx.accounts.market.max_deviation_bps;
    require!(
        (price_diff as u128) * 10000 <= (current_price as u128) * (max_deviation_bps as u128),
        ErrorCode::PriceSlippage
    );

    let order_owner = ctx.accounts.order.owner;
    let order_margin_gns = ctx.accounts.order.margin_gns;
    let order_leverage = ctx.accounts.order.leverage;
    let order_side = ctx.accounts.order.side;
    let order_price_usd_1e6 = ctx.accounts.order.price_usd_1e6;

    require!(ctx.accounts.taker_perp_user.owner == ctx.accounts.taker.key(), ErrorCode::Unauthorized);
    let taker_free = ctx.accounts.taker_perp_user.deposited.checked_sub(ctx.accounts.taker_perp_user.locked_margin).ok_or(ErrorCode::MathOverflow)?;
    require!(taker_free >= order_margin_gns, ErrorCode::InsufficientFreeMargin);
    ctx.accounts.taker_perp_user.locked_margin = ctx.accounts.taker_perp_user.locked_margin.checked_add(order_margin_gns).ok_or(ErrorCode::MathOverflow)?;

    require!(ctx.accounts.maker_perp_user.owner == order_owner, ErrorCode::Unauthorized);
    let maker_nonce = ctx.accounts.maker_perp_user.position_nonce;
    ctx.accounts.maker_perp_user.position_nonce = maker_nonce.checked_add(1).ok_or(ErrorCode::MathOverflow)?;

    let taker_nonce = ctx.accounts.taker_perp_user.position_nonce;
    ctx.accounts.taker_perp_user.position_nonce = taker_nonce.checked_add(1).ok_or(ErrorCode::MathOverflow)?;

    let gns_price = ctx.accounts.gns_usd_oracle.price_usd_1e6;
    let notional = (order_margin_gns as u128)
        .checked_mul(gns_price as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(ONE_TOKEN as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(order_leverage as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let market_key = ctx.accounts.market.key();
    let taker_key = ctx.accounts.taker.key();
    let maker_position_bump = ctx.bumps.maker_position;
    let taker_position_bump = ctx.bumps.taker_position;

    // Read both position keys up front so no field is borrowed both
    // mutably and immutably at the same time further down.
    let maker_position_key = ctx.accounts.maker_position.key();
    let taker_position_key = ctx.accounts.taker_position.key();

    let maker_pos = &mut ctx.accounts.maker_position;
    maker_pos.owner = order_owner;
    maker_pos.market = market_key;
    maker_pos.side = order_side;
    maker_pos.margin_gns = order_margin_gns;
    maker_pos.leverage = order_leverage;
    maker_pos.entry_price_usd_1e6 = order_price_usd_1e6;
    maker_pos.entry_gns_usd_1e6 = gns_price;
    maker_pos.notional_usd_1e6 = notional;
    maker_pos.opened_at = clock.unix_timestamp;
    maker_pos.bump = maker_position_bump;
    maker_pos.counterparty = taker_key;
    maker_pos.counterparty_position = taker_position_key;
    maker_pos.nonce = maker_nonce;
    let maker_owner_for_event = maker_pos.owner;

    let taker_pos = &mut ctx.accounts.taker_position;
    taker_pos.owner = taker_key;
    taker_pos.market = market_key;
    taker_pos.side = 1 - order_side;
    taker_pos.margin_gns = order_margin_gns;
    taker_pos.leverage = order_leverage;
    taker_pos.entry_price_usd_1e6 = order_price_usd_1e6;
    taker_pos.entry_gns_usd_1e6 = gns_price;
    taker_pos.notional_usd_1e6 = notional;
    taker_pos.opened_at = clock.unix_timestamp;
    taker_pos.bump = taker_position_bump;
    taker_pos.counterparty = order_owner;
    taker_pos.counterparty_position = maker_position_key;
    taker_pos.nonce = taker_nonce;
    let taker_owner_for_event = taker_pos.owner;

    let market = &mut ctx.accounts.market;
    if order_side == 0 {
        market.open_interest_long_gns = market.open_interest_long_gns.checked_add(order_margin_gns).ok_or(ErrorCode::MathOverflow)?;
        market.open_interest_short_gns = market.open_interest_short_gns.checked_add(order_margin_gns).ok_or(ErrorCode::MathOverflow)?;
    } else {
        market.open_interest_short_gns = market.open_interest_short_gns.checked_add(order_margin_gns).ok_or(ErrorCode::MathOverflow)?;
        market.open_interest_long_gns = market.open_interest_long_gns.checked_add(order_margin_gns).ok_or(ErrorCode::MathOverflow)?;
    }

    ctx.accounts.order.matched = true;

    emit!(OrderMatched { maker: maker_owner_for_event, taker: taker_owner_for_event, price_usd_1e6: order_price_usd_1e6, margin_gns: order_margin_gns, leverage: order_leverage });
    Ok(())
}

pub fn close_perp_position(ctx: Context<ClosePerpPosition>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let pos = &mut ctx.accounts.position;
    let user = &mut ctx.accounts.user;
    require_keys_eq!(pos.owner, user.owner, ErrorCode::NotPositionOwner);
    require_keys_eq!(pos.owner, ctx.accounts.owner.key(), ErrorCode::Unauthorized);
    require!(pos.margin_gns > 0, ErrorCode::PerpPositionAlreadyClosed);

    let market = &mut ctx.accounts.market;
    let gns_oracle = &ctx.accounts.gns_usd_oracle;
    let clock = Clock::get()?;
    require!(market.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - market.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);

    let current_price = market.price_usd_1e6;
    let (profit, loss) = compute_pnl(pos.side, pos.margin_gns, pos.leverage, pos.entry_price_usd_1e6, current_price, gns_oracle.price_usd_1e6)?;

    user.locked_margin = user.locked_margin.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;

    if profit > 0 {
        user.deposited = user.deposited.checked_add(profit).ok_or(ErrorCode::MathOverflow)?;
    }
    if loss > 0 {
        user.deposited = user.deposited.checked_sub(loss).ok_or(ErrorCode::MathOverflow)?;
    }

    if pos.side == 0 {
        market.open_interest_long_gns = market.open_interest_long_gns.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;
    } else {
        market.open_interest_short_gns = market.open_interest_short_gns.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;
    }

    emit!(PositionClosed {
        owner: pos.owner,
        market: market.key(),
        payout_gns: pos.margin_gns.checked_add(profit).unwrap_or(0),
        margin_gns: pos.margin_gns,
        profit_gns: profit,
        loss_gns: loss,
    });

    Ok(())
}

pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
    require!(!ctx.accounts.config.paused, ErrorCode::EmergencyPaused);
    let pos = &ctx.accounts.position;
    let user = &mut ctx.accounts.user;
    let market = &mut ctx.accounts.market;
    let gns_oracle = &ctx.accounts.gns_usd_oracle;
    let clock = Clock::get()?;
    require!(market.last_updated <= clock.unix_timestamp, ErrorCode::OracleFutureTimestamp);
    require!(clock.unix_timestamp - market.last_updated <= ORACLE_MAX_STALENESS_SECS, ErrorCode::StalePrice);

    require!(pos.margin_gns > 0, ErrorCode::PerpPositionAlreadyClosed);

    let current_price = market.price_usd_1e6;
    let (_, loss) = compute_pnl(pos.side, pos.margin_gns, pos.leverage, pos.entry_price_usd_1e6, current_price, gns_oracle.price_usd_1e6)?;
    let maintenance = pos.margin_gns.checked_mul(MAINTENANCE_MARGIN_BPS).ok_or(ErrorCode::MathOverflow)?.checked_div(10000).ok_or(ErrorCode::MathOverflow)?;
    require!(loss >= maintenance, ErrorCode::LiquidationNotEligible);

    user.locked_margin = user.locked_margin.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;

    let remaining = pos.margin_gns.saturating_sub(loss);
    let bounty = remaining.min(LIQUIDATION_BOUNTY_GNS);

    let total_deduction = loss.checked_add(bounty).ok_or(ErrorCode::MathOverflow)?;
    user.deposited = user.deposited.checked_sub(total_deduction).ok_or(ErrorCode::MathOverflow)?;

    let liquidator_user = &mut ctx.accounts.liquidator_user;
    liquidator_user.deposited = liquidator_user.deposited.checked_add(bounty).ok_or(ErrorCode::MathOverflow)?;

    if pos.side == 0 {
        market.open_interest_long_gns = market.open_interest_long_gns.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;
    } else {
        market.open_interest_short_gns = market.open_interest_short_gns.checked_sub(pos.margin_gns).ok_or(ErrorCode::MathOverflow)?;
    }

    emit!(PositionLiquidated { owner: pos.owner, market: market.key(), liquidator: ctx.accounts.liquidator.key(), payout_gns: bounty });

    Ok(())
}
