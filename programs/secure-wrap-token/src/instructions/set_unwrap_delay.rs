use {crate::state::global_state::GlobalState, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct SetUnwrapDelay<'info> {
    #[account(
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,
}

pub fn set_unwrap_delay(ctx: Context<SetUnwrapDelay>, unwrap_delay_seconds: u64) -> Result<()> {
    ctx.accounts
        .global_state
        .set_unwrap_delay_seconds(unwrap_delay_seconds)
}
