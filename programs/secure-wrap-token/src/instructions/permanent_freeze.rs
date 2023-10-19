use {
    crate::state::{
        global_state::GlobalState, secure_wrap_token_state::SecureWrapTokenState,
        user_token_state::UserTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct PermanentFreeze<'info> {
    #[account(
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>,

    /// CHECK: Separate account for fee-paying, can be same as authority.
    #[account(mut)]
    pub fee_payer: AccountInfo<'info>,

    #[account()]
    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        mut,
        constraint = account_to_freeze.mint == secure_wrap_token_mint.key(),
    )]
    pub account_to_freeze: Box<Account<'info, TokenAccount>>,

    // We require that the original token account already be frozen before SWT can permanently freeze it.
    #[account(
        mut,
        constraint = original_token_account.mint == original_token_mint.key(),
        constraint = original_token_account.owner == account_to_freeze.owner,
    )]
    pub original_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump,
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"user_token_state", secure_wrap_token_mint.key().as_ref(), account_to_freeze.key().as_ref()],
        bump = user_token_state.bump,
    )]
    pub user_token_state: Box<Account<'info, UserTokenState>>,

    #[account(
        mut,
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump,
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn permanent_freeze(ctx: Context<PermanentFreeze>) -> Result<()> {
    ctx.accounts
        .secure_wrap_token_state
        .add_permanently_frozen_tokens(ctx.accounts.account_to_freeze.amount)?;
    ctx.accounts.user_token_state.permanent_freeze(
        &ctx.accounts.original_token_account,
        &ctx.accounts.account_to_freeze,
    )
}
