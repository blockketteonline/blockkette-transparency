use anchor_lang::prelude::*;
use solana_keccak_hasher as keccak;

use crate::constants::*;
use crate::errors::ErrorCode;
use crate::state::bonding_curve::BondingCurve;

// `construct_uint!` internally writes bare `Result` in its generated code,
// expecting the standard library's `core::result::Result<T, E>`. Because most
// files in this crate do `use anchor_lang::prelude::*;`, that glob import
// shadows `Result` with Anchor's single-parameter `Result<T> =
// core::result::Result<T, Error>` alias, which breaks the macro (E0107,
// mismatched Result types, missing methods like `.sqrt()`). Isolating the
// macro in its own module — which does NOT import the Anchor prelude — keeps
// `Result` pointing at the real, two-parameter standard library type, so the
// macro expands correctly.
mod u256_math {
    uint::construct_uint! {
        pub struct U256(4);
    }
}
pub use u256_math::U256;

// `construct_uint!` never generates a `.sqrt()` method, and the
// `integer-sqrt` crate's `IntegerSquareRoot` trait only covers Rust's
// built-in integer types, not custom big-integer types like `U256`. This is
// a standard Newton's-method (Babylonian) integer square root that works
// with any type supporting the basic arithmetic `U256` already has.
pub fn u256_sqrt(n: U256) -> U256 {
    if n.is_zero() {
        return U256::zero();
    }
    let mut x = n;
    let mut y = (x + U256::one()) >> 1;
    while y < x {
        x = y;
        y = (x + n / x) >> 1;
    }
    x
}

pub fn get_current_price(state: &BondingCurve) -> Result<u64> {
    let whole_supply = state.total_supply / ONE_TOKEN;
    let price: u128 = (whole_supply as u128)
        .checked_mul(state.price_increment as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_add(state.base_price as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    u64::try_from(price).map_err(|_| ErrorCode::MathOverflow.into())
}

pub fn bonding_curve_buy(state: &mut BondingCurve, sol_amount: u64) -> Result<u64> {
    require!(sol_amount > 0, ErrorCode::ZeroAmount);
    let s = state.total_supply as u128;
    let base = state.base_price as u128;
    let inc  = state.price_increment as u128;
    let one  = ONE_TOKEN as u128;

    if inc == 0 {
        let dx = (sol_amount as u128)
            .checked_mul(one).ok_or(ErrorCode::MathOverflow)?
            .checked_div(base).ok_or(ErrorCode::MathOverflow)?;
        let raw_tokens = u64::try_from(dx).map_err(|_| ErrorCode::MathOverflow)?;
        require!(raw_tokens > 0, ErrorCode::ZeroAmount);

        if state.total_supply > 0 {
            let max_trade = state.total_supply
                .checked_mul(MAX_TRADE_FRACTION_BPS).ok_or(ErrorCode::MathOverflow)?
                .checked_div(10000).ok_or(ErrorCode::MathOverflow)?;
            require!(raw_tokens <= max_trade, ErrorCode::TradeSizeTooLarge);
        }

        let new_supply = state.total_supply.checked_add(raw_tokens).ok_or(ErrorCode::MathOverflow)?;
        require!(new_supply <= state.max_supply, ErrorCode::MaxSupplyExceeded);
        state.total_supply = new_supply;
        state.total_sol_raised = state.total_sol_raised.checked_add(sol_amount).ok_or(ErrorCode::MathOverflow)?;
        return Ok(raw_tokens);
    }

    let a = U256::from(inc);
    let b = {
        let part1 = U256::from(base) * U256::from(2) * U256::from(one);
        let part2 = U256::from(inc) * U256::from(2) * U256::from(s);
        part1.checked_add(part2).ok_or(ErrorCode::MathOverflow)?
    };
    let c = {
        let two_one_sq = U256::from(2) * U256::from(one) * U256::from(one);
        two_one_sq.checked_mul(U256::from(sol_amount)).ok_or(ErrorCode::MathOverflow)?
    };
    let disc = {
        let b_sq = b.checked_mul(b).ok_or(ErrorCode::MathOverflow)?;
        let four_ac = U256::from(4) * a.checked_mul(c).ok_or(ErrorCode::MathOverflow)?;
        b_sq.checked_add(four_ac).ok_or(ErrorCode::MathOverflow)?
    };
    let sqrt_disc = u256_sqrt(disc);
    let numerator = if sqrt_disc > b {
        sqrt_disc.checked_sub(b).ok_or(ErrorCode::MathOverflow)?
    } else {
        return Err(ErrorCode::MathOverflow.into());
    };
    let denominator = U256::from(2) * a;
    let dx = numerator.checked_div(denominator).ok_or(ErrorCode::MathOverflow)?;

    let dx_u128: u128 = dx.try_into().map_err(|_| ErrorCode::MathOverflow)?;
    let raw_tokens = u64::try_from(dx_u128).map_err(|_| ErrorCode::MathOverflow)?;
    require!(raw_tokens > 0, ErrorCode::ZeroAmount);

    if state.total_supply > 0 {
        let max_trade = state.total_supply
            .checked_mul(MAX_TRADE_FRACTION_BPS).ok_or(ErrorCode::MathOverflow)?
            .checked_div(10000).ok_or(ErrorCode::MathOverflow)?;
        require!(raw_tokens <= max_trade, ErrorCode::TradeSizeTooLarge);
    }

    let new_supply = state.total_supply.checked_add(raw_tokens).ok_or(ErrorCode::MathOverflow)?;
    require!(new_supply <= state.max_supply, ErrorCode::MaxSupplyExceeded);

    state.total_supply = new_supply;
    state.total_sol_raised = state.total_sol_raised.checked_add(sol_amount).ok_or(ErrorCode::MathOverflow)?;
    Ok(raw_tokens)
}

pub fn bonding_curve_sell(state: &mut BondingCurve, token_amount: u64) -> Result<u64> {
    require!(token_amount > 0, ErrorCode::ZeroAmount);
    require!(token_amount <= state.total_supply, ErrorCode::InsufficientSupply);

    if state.total_supply > 0 {
        let max_trade = state.total_supply
            .checked_mul(MAX_TRADE_FRACTION_BPS).ok_or(ErrorCode::MathOverflow)?
            .checked_div(10000).ok_or(ErrorCode::MathOverflow)?;
        require!(token_amount <= max_trade, ErrorCode::TradeSizeTooLarge);
    }

    let sol_u64 = curve_sell_proceeds_readonly(state, token_amount)?;
    state.total_supply = state.total_supply.checked_sub(token_amount).ok_or(ErrorCode::MathOverflow)?;
    state.total_sol_raised = state.total_sol_raised.checked_sub(sol_u64).ok_or(ErrorCode::MathOverflow)?;
    Ok(sol_u64)
}

pub fn curve_sell_proceeds_readonly(state: &BondingCurve, token_amount: u64) -> Result<u64> {
    require!(token_amount > 0, ErrorCode::ZeroAmount);
    require!(token_amount <= state.total_supply, ErrorCode::InsufficientSupply);

    let s = U256::from(state.total_supply);
    let dx = U256::from(token_amount);
    let base = U256::from(state.base_price);
    let inc = U256::from(state.price_increment);
    let one = U256::from(ONE_TOKEN);

    let sol_u256 = if state.price_increment == 0 {
        base.checked_mul(dx).ok_or(ErrorCode::MathOverflow)?
            .checked_div(one).ok_or(ErrorCode::MathOverflow)?
    } else {
        let term1 = base.checked_mul(dx).ok_or(ErrorCode::MathOverflow)?
            .checked_div(one).ok_or(ErrorCode::MathOverflow)?;
        let two_s_dx = U256::from(2).checked_mul(s).ok_or(ErrorCode::MathOverflow)?
            .checked_mul(dx).ok_or(ErrorCode::MathOverflow)?;
        let dx_sq = dx.checked_mul(dx).ok_or(ErrorCode::MathOverflow)?;
        let par = two_s_dx.checked_sub(dx_sq).ok_or(ErrorCode::MathOverflow)?;
        let term2_num = inc.checked_mul(par).ok_or(ErrorCode::MathOverflow)?;
        let term2_den = U256::from(2).checked_mul(one).ok_or(ErrorCode::MathOverflow)?
            .checked_mul(one).ok_or(ErrorCode::MathOverflow)?;
        let term2 = term2_num.checked_div(term2_den).ok_or(ErrorCode::MathOverflow)?;
        term1.checked_add(term2).ok_or(ErrorCode::MathOverflow)?
    };

    u64::try_from(sol_u256).map_err(|_| ErrorCode::MathOverflow.into())
}

pub fn usd_to_gns(usd_cents: u64, gns_price_cents: u64) -> Result<u64> {
    let scaled = (usd_cents as u128)
        .checked_mul(ONE_TOKEN as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(gns_price_cents as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    u64::try_from(scaled).map_err(|_| ErrorCode::MathOverflow.into())
}

pub fn gns_raw_to_usd_1e6(raw_amount: u64, gns_usd_1e6: u64) -> Result<u128> {
    (raw_amount as u128)
        .checked_mul(gns_usd_1e6 as u128).ok_or(ErrorCode::MathOverflow)?
        .checked_div(ONE_TOKEN as u128).ok_or(ErrorCode::MathOverflow.into())
}

pub fn compute_pnl(
    side: u8,
    margin_gns: u64,
    leverage: u16,
    entry_price: u64,
    current_price: u64,
    gns_usd_1e6: u64,
) -> Result<(u64, u64)> {
    let notional_usd_1e6 = (margin_gns as u128)
        .checked_mul(gns_usd_1e6 as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(ONE_TOKEN as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(leverage as u128)
        .ok_or(ErrorCode::MathOverflow)?;

    let price_move_bps: i128 = if side == 0 {
        (current_price as i128 - entry_price as i128)
            .checked_mul(1_000_000)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(entry_price as i128)
            .ok_or(ErrorCode::MathOverflow)?
    } else {
        (entry_price as i128 - current_price as i128)
            .checked_mul(1_000_000)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(entry_price as i128)
            .ok_or(ErrorCode::MathOverflow)?
    };

    let pnl_usd_1e6: i128 = (notional_usd_1e6 as i128)
        .checked_mul(price_move_bps)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(1_000_000)
        .ok_or(ErrorCode::MathOverflow)?;

    let pnl_gns: i128 = pnl_usd_1e6
        .checked_mul(ONE_TOKEN as i128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(gns_usd_1e6 as i128)
        .ok_or(ErrorCode::MathOverflow)?;

    let margin = margin_gns as i128;
    let payout = (margin + pnl_gns).max(0);

    if payout > margin {
        Ok(((payout - margin) as u64, 0))
    } else {
        Ok((0, (margin - payout) as u64))
    }
}

pub fn compute_market_cap_usd_1e6(
    curve: &BondingCurve,
    sol_usd_1e6: u64,
) -> Result<u128> {
    let current_price_lamports = get_current_price(curve)?;
    let total_supply = curve.total_supply as u128;
    let numerator = total_supply
        .checked_mul(current_price_lamports as u128)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_mul(sol_usd_1e6 as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let denominator = (LAMPORTS_PER_SOL as u128)
        .checked_mul(ONE_TOKEN as u128)
        .ok_or(ErrorCode::MathOverflow)?;
    let market_cap = numerator
        .checked_div(denominator)
        .ok_or(ErrorCode::MathOverflow)?;
    Ok(market_cap)
}

pub fn verify_merkle_proof(
    leaf: [u8; 32],
    proof: Vec<[u8; 32]>,
    root: [u8; 32],
) -> bool {
    let mut hash = leaf;
    for p in proof {
        let mut combined = [0u8; 64];
        if hash <= p {
            combined[..32].copy_from_slice(&hash);
            combined[32..].copy_from_slice(&p);
        } else {
            combined[..32].copy_from_slice(&p);
            combined[32..].copy_from_slice(&hash);
        }
        hash = keccak::hashv(&[&combined]).to_bytes();
    }
    hash == root
}
