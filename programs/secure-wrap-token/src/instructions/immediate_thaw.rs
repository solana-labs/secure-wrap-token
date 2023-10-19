use {
    crate::state::{
        global_state::GlobalState, secure_wrap_token_state::SecureWrapTokenState,
        user_token_state::UserTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct ImmediateThaw<'info> {
    #[account(
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>, // The authority is able to immediately thaw any non-permanent frozen account.

    /// CHECK: Separate account for fee-paying, can be same as authority.
    #[account(mut)]
    pub fee_payer: AccountInfo<'info>,

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

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn immediate_thaw(ctx: Context<ImmediateThaw>) -> Result<()> {
    let user_token_state = &mut ctx.accounts.user_token_state;
    user_token_state.immediate_thaw(&ctx.accounts.account_to_thaw)?;
    ctx.accounts.secure_wrap_token_state.thaw_account(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        &ctx.accounts.account_to_thaw,
        user_token_state,
    )
}
