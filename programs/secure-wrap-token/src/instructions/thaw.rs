use {
    crate::state::{
        secure_wrap_token_state::SecureWrapTokenState, user_token_state::UserTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Thaw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // Anyone can thaw any account once its freeze period has passed.

    #[account()]
    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        mut,
        constraint = account_to_thaw.mint == secure_wrap_token_mint.key(),
    )]
    pub account_to_thaw: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump,
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"user_token_state", secure_wrap_token_mint.key().as_ref(), account_to_thaw.key().as_ref()],
        bump = user_token_state.bump,
    )]
    pub user_token_state: Box<Account<'info, UserTokenState>>,

    #[account(
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump,
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn thaw(ctx: Context<Thaw>) -> Result<()> {
    ctx.accounts.secure_wrap_token_state.thaw_account(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        &ctx.accounts.account_to_thaw,
        &mut ctx.accounts.user_token_state,
    )
}
