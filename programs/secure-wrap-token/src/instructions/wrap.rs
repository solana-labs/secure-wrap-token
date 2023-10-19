use {
    crate::{
        error::SecureWrapTokenError, state::secure_wrap_token_state::SecureWrapTokenState,
        utils::user_utils::transfer_tokens,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Wrap<'info> {
    // Owner is the user requesting to wrap their tokens.
    #[account()]
    pub owner: Signer<'info>,

    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        mut,
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

    // This program's token account for the original token mint.
    #[account(
        mut,
        seeds = [b"program_original_token_account", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.program_original_token_account_bump
    )]
    pub program_original_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    token_program: Program<'info, Token>,
}

pub fn wrap(ctx: Context<Wrap>, amount: u64) -> Result<()> {
    require_neq!(amount, 0, SecureWrapTokenError::InvalidTokenAmount);

    // Send the user's token to this program.
    transfer_tokens(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_account.to_account_info(),
        ctx.accounts
            .program_original_token_account
            .to_account_info(),
        ctx.accounts.owner.to_account_info(),
        amount,
    )?;
    // Mint equal amount of wrapped tokens for the user.
    ctx.accounts.secure_wrap_token_state.mint_wrapped_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        ctx.accounts.secure_wrap_token_account.to_account_info(),
        amount,
    )
}
