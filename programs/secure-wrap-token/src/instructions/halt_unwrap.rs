use {crate::state::global_state::GlobalState, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct HaltUnwrap<'info> {
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

pub fn halt_unwrap(ctx: Context<HaltUnwrap>) -> Result<()> {
    ctx.accounts.global_state.halt_unwrap();
    Ok(())
}
