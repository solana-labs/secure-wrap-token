use {
    crate::state::{
        global_state::GlobalState, secure_wrap_token_state::SecureWrapTokenState,
        user_token_state::UserTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct Freeze<'info> {
    #[account(
        mut,
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

    #[account(
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump,
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = fee_payer,
        space = 8 + UserTokenState::INIT_SPACE,
        seeds = [b"user_token_state", secure_wrap_token_mint.key().as_ref(), account_to_freeze.key().as_ref()],
        bump,
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

pub fn freeze(ctx: Context<Freeze>, freeze_period_seconds: u64) -> Result<()> {
    // Each call to freeze will overwrite the thaw time from previous freeze on the user.
    let user_token_state = &mut ctx.accounts.user_token_state;
    user_token_state.validate_initialization(
        ctx.accounts.account_to_freeze.key(),
        ctx.bumps.user_token_state,
    )?;
    user_token_state.freeze(&ctx.accounts.account_to_freeze, freeze_period_seconds)?;
    ctx.accounts.secure_wrap_token_state.freeze_account(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        ctx.accounts.account_to_freeze.to_account_info(),
    )
}
