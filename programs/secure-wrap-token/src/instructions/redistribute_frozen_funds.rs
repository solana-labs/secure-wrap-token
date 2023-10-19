use {
    crate::{
        error::SecureWrapTokenError,
        state::{
            global_state::GlobalState, secure_wrap_token_state::SecureWrapTokenState,
            user_token_state::UserTokenState,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct RedistributeFrozenFunds<'info> {
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
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump,
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = permanently_frozen_account.mint == secure_wrap_token_mint.key(),
    )]
    pub permanently_frozen_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = receiving_account.mint == secure_wrap_token_mint.key(),
    )]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"user_token_state", secure_wrap_token_mint.key().as_ref(), permanently_frozen_account.key().as_ref()],
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

pub fn redistribute_frozen_funds(ctx: Context<RedistributeFrozenFunds>, amount: u64) -> Result<()> {
    require_neq!(amount, 0, SecureWrapTokenError::InvalidTokenAmount);
    ctx.accounts
        .user_token_state
        .record_frozen_fund_distribution(&ctx.accounts.permanently_frozen_account, amount)?;
    ctx.accounts
        .secure_wrap_token_state
        .add_redistributed_tokens(amount)?;

    // Mint wrapped tokens to the receiving_account, representing a token redistribution from the permanently frozen account.
    ctx.accounts.secure_wrap_token_state.mint_wrapped_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        ctx.accounts.receiving_account.to_account_info(),
        amount,
    )
}
