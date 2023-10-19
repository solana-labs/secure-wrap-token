use {crate::state::global_state::GlobalState, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct ResumeWrapOrders<'info> {
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

pub fn resume_wrap_orders(ctx: Context<ResumeWrapOrders>) -> Result<()> {
    ctx.accounts.global_state.resume_wrap_orders();
    Ok(())
}
