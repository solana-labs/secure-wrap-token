use {
    crate::state::{
        global_state::GlobalState, pending_unwrap::PendingUnwrap,
        secure_wrap_token_state::SecureWrapTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct CancelUnwrap<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        constraint = secure_wrap_token_mint.key() == pending_unwrap.secure_wrap_token_mint,
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = secure_wrap_token_account.mint == secure_wrap_token_mint.key(),
        constraint = secure_wrap_token_account.key() == pending_unwrap.secure_wrap_token_account,
        has_one = owner,
    )]
    pub secure_wrap_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"pending_unwrap", original_token_mint.key().as_ref(), owner.key().as_ref()],
        bump = pending_unwrap.bump,
        close = owner,
        has_one = owner,
    )]
    pub pending_unwrap: Box<Account<'info, PendingUnwrap>>,

    #[account(
        mut,
        seeds = [b"program_wrapped_token_account", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.program_wrapped_token_account_bump,
    )]
    pub program_wrapped_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn cancel_unwrap(ctx: Context<CancelUnwrap>) -> Result<()> {
    ctx.accounts.global_state.validate_unwrap_allowed()?;
    ctx.accounts.secure_wrap_token_state.send_wrapped_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_account.to_account_info(),
        ctx.accounts.program_wrapped_token_account.to_account_info(),
        ctx.accounts.pending_unwrap.amount,
    )
}
