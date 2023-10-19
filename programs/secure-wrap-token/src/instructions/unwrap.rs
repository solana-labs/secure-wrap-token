use {
    crate::{
        error::SecureWrapTokenError,
        state::{
            global_state::GlobalState, pending_unwrap::PendingUnwrap,
            secure_wrap_token_state::SecureWrapTokenState,
        },
        utils::user_utils::transfer_tokens,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Unwrap<'info> {
    // Owner is the user requesting to unwrap their tokens.
    // This instruction creates a 'pending_unwrap' where the funds release after a delay period.
    #[account(mut)]
    pub owner: Signer<'info>,

    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = token_account.mint == original_token_mint.key(),
        has_one = owner,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = secure_wrap_token_account.mint == secure_wrap_token_mint.key(),
        has_one = owner,
    )]
    pub secure_wrap_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = owner,
        space = 8 + PendingUnwrap::INIT_SPACE,
        seeds = [b"pending_unwrap", original_token_mint.key().as_ref(), owner.key().as_ref()],
        bump,
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
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn unwrap(ctx: Context<Unwrap>, amount: u64) -> Result<()> {
    require_neq!(amount, 0, SecureWrapTokenError::InvalidTokenAmount);
    ctx.accounts.global_state.validate_unwrap_allowed()?;

    // Transfer user's wrapped tokens into escrow.
    transfer_tokens(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_account.to_account_info(),
        ctx.accounts.program_wrapped_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        amount,
    )?;
    ctx.accounts.pending_unwrap.initialize(
        ctx.accounts.owner.key(),
        ctx.accounts.secure_wrap_token_mint.key(),
        ctx.accounts.secure_wrap_token_account.key(),
        ctx.accounts.token_account.key(),
        ctx.accounts
            .global_state
            .compute_unwrap_release_timestamp()
            .unwrap(),
        amount,
        *ctx.bumps
            .get("pending_unwrap")
            .ok_or(ProgramError::InvalidSeeds)?,
    );
    Ok(())
}
