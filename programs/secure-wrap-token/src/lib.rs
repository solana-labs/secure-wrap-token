#![allow(clippy::result_large_err)]

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use {anchor_lang::prelude::*, instructions::*};

solana_security_txt::security_txt! {
    name: "Secure Wrap Token",
    project_url: "https://github.com/solana-labs/secure-wrap-token",
    contacts: "email:defi@solana.com",
    policy: "",
    preferred_languages: "en",
    auditors: ""
}

declare_id!("NNcz9dDJ5cSxeNy95kn4AosZaQ3jzBzMzrhxyRzDyXQ");

#[program]
pub mod secure_wrap_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn create_secure_wrapped_token(ctx: Context<CreateSecureWrappedToken>) -> Result<()> {
        instructions::create_secure_wrapped_token(ctx)
    }

    pub fn wrap(ctx: Context<Wrap>, amount: u64) -> Result<()> {
        instructions::wrap(ctx, amount)
    }

    pub fn unwrap(ctx: Context<Unwrap>, amount: u64) -> Result<()> {
        instructions::unwrap(ctx, amount)
    }

    pub fn freeze(ctx: Context<Freeze>, freeze_period_seconds: u64) -> Result<()> {
        instructions::freeze(ctx, freeze_period_seconds)
    }

    pub fn thaw(ctx: Context<Thaw>) -> Result<()> {
        instructions::thaw(ctx)
    }

    pub fn immediate_thaw(ctx: Context<ImmediateThaw>) -> Result<()> {
        instructions::immediate_thaw(ctx)
    }

    pub fn permanent_freeze(ctx: Context<PermanentFreeze>) -> Result<()> {
        instructions::permanent_freeze(ctx)
    }

    pub fn place_order(ctx: Context<PlaceOrder>, params: PlaceOrderParams) -> Result<()> {
        instructions::place_order(ctx, &params)
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, params: CancelOrderParams) -> Result<()> {
        instructions::cancel_order(ctx, &params)
    }

    pub fn fill_order(ctx: Context<FillOrder>, params: FillOrderParams) -> Result<()> {
        instructions::fill_order(ctx, &params)
    }

    pub fn fill_order_program(
        ctx: Context<FillOrderProgram>,
        params: FillOrderProgramParams,
    ) -> Result<()> {
        instructions::fill_order_program(ctx, &params)
    }

    pub fn distribute_frozen_funds(
        ctx: Context<RedistributeFrozenFunds>,
        amount: u64,
    ) -> Result<()> {
        instructions::redistribute_frozen_funds(ctx, amount)
    }

    pub fn halt_orders(ctx: Context<HaltOrders>) -> Result<()> {
        instructions::halt_orders(ctx)
    }

    pub fn resume_orders(ctx: Context<ResumeOrders>) -> Result<()> {
        instructions::resume_orders(ctx)
    }

    pub fn set_unwrap_delay(ctx: Context<SetUnwrapDelay>, unwrap_delay_seconds: u64) -> Result<()> {
        instructions::set_unwrap_delay(ctx, unwrap_delay_seconds)
    }

    pub fn release_unwrap(ctx: Context<ReleaseUnwrap>) -> Result<()> {
        instructions::release_unwrap(ctx)
    }

    pub fn cancel_unwrap(ctx: Context<CancelUnwrap>) -> Result<()> {
        instructions::cancel_unwrap(ctx)
    }

    pub fn halt_unwrap(ctx: Context<HaltUnwrap>) -> Result<()> {
        instructions::halt_unwrap(ctx)
    }

    pub fn resume_unwrap(ctx: Context<ResumeUnwrap>) -> Result<()> {
        instructions::resume_unwrap(ctx)
    }

    pub fn halt_wrap_orders(ctx: Context<HaltWrapOrders>) -> Result<()> {
        instructions::halt_wrap_orders(ctx)
    }

    pub fn resume_wrap_orders(ctx: Context<ResumeWrapOrders>) -> Result<()> {
        instructions::resume_wrap_orders(ctx)
    }

    pub fn cancel_unwrap_program(ctx: Context<CancelUnwrapProgram>) -> Result<()> {
        instructions::cancel_unwrap_program(ctx)
    }

    pub fn cancel_order_program(
        ctx: Context<CancelOrderProgram>,
        params: CancelOrderProgramParams,
    ) -> Result<()> {
        instructions::cancel_order_program(ctx, &params)
    }
}
