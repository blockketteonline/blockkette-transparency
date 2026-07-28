use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod math;
pub mod state;
pub mod instructions;

use instructions::*;
use state::Tier;

declare_id!("4xx1mSeusFLGN9oHoBS1DfGcDSfdTdeRaPheWqU8zZSZ");

#[program]
pub mod genesis_token {
    use super::*;

    // ===================== PROTOCOL ADMIN =====================
    pub fn initialize_protocol_admin(ctx: Context<InitializeProtocolAdmin>) -> Result<()> {
        instructions::protocol_admin::initialize_protocol_admin(ctx)
    }

    pub fn nominate_protocol_admin(ctx: Context<NominateProtocolAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::protocol_admin::nominate_protocol_admin(ctx, new_admin)
    }

    pub fn accept_protocol_admin(ctx: Context<AcceptProtocolAdmin>) -> Result<()> {
        instructions::protocol_admin::accept_protocol_admin(ctx)
    }

    pub fn cancel_protocol_admin_transfer(ctx: Context<CancelProtocolAdminTransfer>) -> Result<()> {
        instructions::protocol_admin::cancel_protocol_admin_transfer(ctx)
    }

    // ===================== BONDING CURVE =====================
    pub fn initialize(ctx: Context<Initialize>, base_price: u64, price_inc: u64, max_supply: u64) -> Result<()> {
        instructions::bonding_curve::initialize(ctx, base_price, price_inc, max_supply)
    }

    pub fn buy(ctx: Context<Buy>, amount: u64, min_tokens_out: u64) -> Result<()> {
        instructions::bonding_curve::buy(ctx, amount, min_tokens_out)
    }

    pub fn sell(ctx: Context<Sell>, amount: u64, min_sol_out: u64) -> Result<()> {
        instructions::bonding_curve::sell(ctx, amount, min_sol_out)
    }

    pub fn nominate_admin(ctx: Context<NominateAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::bonding_curve::nominate_admin(ctx, new_admin)
    }

    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        instructions::bonding_curve::accept_admin(ctx)
    }

    pub fn cancel_admin_transfer(ctx: Context<CancelAdminTransfer>) -> Result<()> {
        instructions::bonding_curve::cancel_admin_transfer(ctx)
    }

    pub fn update_curve_params(ctx: Context<UpdateAdmin>, new_base_price: Option<u64>, new_price_increment: Option<u64>, new_max_supply: Option<u64>) -> Result<()> {
        instructions::bonding_curve::update_curve_params(ctx, new_base_price, new_price_increment, new_max_supply)
    }

    pub fn execute_curve_params_update(ctx: Context<UpdateAdmin>) -> Result<()> {
        instructions::bonding_curve::execute_curve_params_update(ctx)
    }

    pub fn cancel_curve_params_update(ctx: Context<UpdateAdmin>) -> Result<()> {
        instructions::bonding_curve::cancel_curve_params_update(ctx)
    }

    // ===================== FACTORY CONFIG =====================
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
        instructions::factory::initialize_factory_config(
            ctx,
            gns_usd_price_cents,
            default_base_price,
            default_price_increment,
            pos_activation_cost_gns,
            ai_subscription_cost_gns,
            ai_subscription_period_secs,
            tiers,
        )
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
        instructions::factory::update_factory_config(
            ctx,
            gns_usd_price_cents,
            default_base_price,
            default_price_increment,
            pos_activation_cost_gns,
            ai_subscription_cost_gns,
            ai_subscription_period_secs,
            tiers,
        )
    }

    pub fn pause_protocol(ctx: Context<PauseProtocol>) -> Result<()> {
        instructions::factory::pause_protocol(ctx)
    }

    pub fn unpause_protocol(ctx: Context<UnpauseProtocol>) -> Result<()> {
        instructions::factory::unpause_protocol(ctx)
    }

    // ===================== BUSINESS TOKEN FACTORY =====================
    pub fn create_business_token(ctx: Context<CreateBusinessToken>, tier_index: u8) -> Result<()> {
        instructions::business_token::create_business_token(ctx, tier_index)
    }

    pub fn upgrade_business_token(ctx: Context<UpgradeBusinessToken>, tier_index: u8) -> Result<()> {
        instructions::business_token::upgrade_business_token(ctx, tier_index)
    }

    pub fn update_business_curve_params(ctx: Context<UpdateBusinessCurveParams>, new_base_price: Option<u64>, new_price_increment: Option<u64>, new_max_supply: Option<u64>) -> Result<()> {
        instructions::business_token::update_business_curve_params(ctx, new_base_price, new_price_increment, new_max_supply)
    }

    // ===================== POS =====================
    pub fn activate_pos(ctx: Context<ActivatePOS>) -> Result<()> {
        instructions::pos::activate_pos(ctx)
    }

    pub fn process_pos_payment(ctx: Context<ProcessPOSPayment>, sol_payment: u64) -> Result<()> {
        instructions::pos::process_pos_payment(ctx, sol_payment)
    }

    // ===================== AI SUBSCRIPTION =====================
    pub fn purchase_ai_subscription(ctx: Context<PurchaseAISubscription>) -> Result<()> {
        instructions::subscription::purchase_ai_subscription(ctx)
    }

    // ===================== PERP VAULT =====================
    pub fn initialize_perp_vault(ctx: Context<InitializePerpVault>) -> Result<()> {
        instructions::perp_vault::initialize_perp_vault(ctx)
    }

    pub fn perp_deposit(ctx: Context<PerpDeposit>, amount: u64) -> Result<()> {
        instructions::perp_vault::perp_deposit(ctx, amount)
    }

    pub fn perp_withdraw(ctx: Context<PerpWithdraw>, amount: u64) -> Result<()> {
        instructions::perp_vault::perp_withdraw(ctx, amount)
    }

    // ===================== GNS STAKING =====================
    pub fn init_staking_pool(ctx: Context<InitStakingPool>) -> Result<()> {
        instructions::staking::init_staking_pool(ctx)
    }

    pub fn stake_gns(ctx: Context<StakeGns>, amount: u64) -> Result<()> {
        instructions::staking::stake_gns(ctx, amount)
    }

    pub fn unstake_gns(ctx: Context<UnstakeGns>, share_amount: u64) -> Result<()> {
        instructions::staking::unstake_gns(ctx, share_amount)
    }

    pub fn sweep_donations(ctx: Context<SweepDonations>) -> Result<()> {
        instructions::staking::sweep_donations(ctx)
    }

    // ===================== ORACLES =====================
    pub fn initialize_sol_usd_oracle(ctx: Context<InitializeSolUsdOracle>, initial_price_usd_1e6: u64) -> Result<()> {
        instructions::oracle::initialize_sol_usd_oracle(ctx, initial_price_usd_1e6)
    }

    pub fn update_sol_usd_price(ctx: Context<UpdateSolUsdPrice>, price_usd_1e6: u64) -> Result<()> {
        instructions::oracle::update_sol_usd_price(ctx, price_usd_1e6)
    }

    pub fn initialize_gns_usd_oracle(ctx: Context<InitializeGnsUsdOracle>, initial_price_usd_1e6: u64) -> Result<()> {
        instructions::oracle::initialize_gns_usd_oracle(ctx, initial_price_usd_1e6)
    }

    pub fn update_gns_usd_price(ctx: Context<UpdateGnsUsdPrice>, price_usd_1e6: u64) -> Result<()> {
        instructions::oracle::update_gns_usd_price(ctx, price_usd_1e6)
    }

    // ===================== PERP MARKETS =====================
    pub fn create_perp_market(
        ctx: Context<CreatePerpMarket>,
        symbol: [u8; 16],
        max_leverage: u16,
        taker_fee_bps: u16,
        initial_price_usd_1e6: u64,
        max_deviation_bps: u16,
        price_delay_secs: i64,
    ) -> Result<()> {
        instructions::perp_market::create_perp_market(
            ctx,
            symbol,
            max_leverage,
            taker_fee_bps,
            initial_price_usd_1e6,
            max_deviation_bps,
            price_delay_secs,
        )
    }

    pub fn create_perp_order(
        ctx: Context<CreatePerpOrder>,
        symbol: [u8; 16],
        side: u8,
        margin_gns: u64,
        leverage: u16,
        price_usd_1e6: u64,
    ) -> Result<()> {
        instructions::perp_market::create_perp_order(ctx, symbol, side, margin_gns, leverage, price_usd_1e6)
    }

    pub fn take_perp_order(ctx: Context<TakePerpOrder>) -> Result<()> {
        instructions::perp_market::take_perp_order(ctx)
    }

    pub fn close_perp_position(ctx: Context<ClosePerpPosition>) -> Result<()> {
        instructions::perp_market::close_perp_position(ctx)
    }

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        instructions::perp_market::liquidate_position(ctx)
    }

    // ===================== AIRDROP =====================
    pub fn init_airdrop(
        ctx: Context<InitAirdrop>,
        merkle_root: [u8; 32],
        airdrop_amount: u64,
        required_market_cap_usd_1e6: u64,
    ) -> Result<()> {
        instructions::airdrop::init_airdrop(ctx, merkle_root, airdrop_amount, required_market_cap_usd_1e6)
    }

    pub fn claim_airdrop(ctx: Context<ClaimAirdrop>, amount: u64, proof: Vec<[u8; 32]>) -> Result<()> {
        instructions::airdrop::claim_airdrop(ctx, amount, proof)
    }

    // ===================== USDT SWAP =====================
    pub fn init_usdt_swap_config(ctx: Context<InitUsdtSwapConfig>, usdt_mint: Pubkey) -> Result<()> {
        instructions::usdt_swap::init_usdt_swap_config(ctx, usdt_mint)
    }

    pub fn swap_gns_for_usdt(ctx: Context<SwapGnsForUsdt>, gns_amount: u64, min_usdt_out: u64) -> Result<()> {
        instructions::usdt_swap::swap_gns_for_usdt(ctx, gns_amount, min_usdt_out)
    }
}
