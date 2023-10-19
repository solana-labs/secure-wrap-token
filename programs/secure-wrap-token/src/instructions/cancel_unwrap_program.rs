use {
    crate::{
        error::SecureWrapTokenError,
        state::{
            global_state::GlobalState, pending_unwrap::PendingUnwrap,
            secure_wrap_token_state::SecureWrapTokenState, user_token_state::UserTokenState,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct CancelUnwrapProgram<'info> {
    // When a pending unwrap belongs to a permanently frozen account, the authority can cancel it.
    // This cancellation temporarily unfreezes the account, moves tokens into it, and then refreezes it.
    #[account(
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>,

    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        constraint = secure_wrap_token_mint.key() == pending_unwrap.secure_wrap_token_mint,
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    /// CHECK: permanently frozen user who owns the pending unwrap to cancel.
    #[account(mut)]
    pub permanently_frozen_user: AccountInfo<'info>,

    #[account(
        mut,
        constraint = secure_wrap_token_account.mint == secure_wrap_token_mint.key(),
        constraint = secure_wrap_token_account.key() == pending_unwrap.secure_wrap_token_account,
        constraint = secure_wrap_token_account.owner == permanently_frozen_user.key(),
    )]
    pub secure_wrap_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = pending_unwrap.owner == permanently_frozen_user.key(),
        seeds = [b"pending_unwrap", original_token_mint.key().as_ref(), permanently_frozen_user.key().as_ref()],
        bump = pending_unwrap.bump,
        close = permanently_frozen_user,
    )]
    pub pending_unwrap: Box<Account<'info, PendingUnwrap>>,

    #[account(
        mut,
        seeds = [b"program_wrapped_token_account", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.program_wrapped_token_account_bump,
    )]
    pub program_wrapped_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    #[account(
        seeds = [b"user_token_state", secure_wrap_token_mint.key().as_ref(), secure_wrap_token_account.key().as_ref()],
        bump = user_token_state.bump,
    )]
    pub user_token_state: Box<Account<'info, UserTokenState>>,

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn cancel_unwrap_program(ctx: Context<CancelUnwrapProgram>) -> Result<()> {
    require!(
        ctx.accounts.user_token_state.permanently_frozen,
        SecureWrapTokenError::ProgramCancelUnwrapError
    );
    ctx.accounts
        .secure_wrap_token_state
        .send_swt_to_frozen_account(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.secure_wrap_token_mint.to_account_info(),
            ctx.accounts.program_wrapped_token_account.to_account_info(),
            ctx.accounts.secure_wrap_token_account.to_account_info(),
            ctx.accounts.pending_unwrap.amount,
        )
}
