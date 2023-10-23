use {crate::state::global_state::GlobalState, anchor_lang::prelude::*};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account()]
    pub authority: Signer<'info>,

    /// CHECK: Separate account for fee-paying, can be same as authority.
    #[account(mut)]
    pub fee_payer: AccountInfo<'info>,

    #[account(
        init,
        payer = fee_payer,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        address = crate::ID,
        constraint = secure_wrap_token_program.programdata_address()? == Some(secure_wrap_token_program_data.key()),
    )]
    pub secure_wrap_token_program: Program<'info, GlobalState>,

    #[account(constraint = secure_wrap_token_program_data.upgrade_authority_address == Some(authority.key()))]
    pub secure_wrap_token_program_data: Box<Account<'info, ProgramData>>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts
        .global_state
        .initialize(ctx.accounts.authority.key(), ctx.bumps.global_state);
    Ok(())
}
