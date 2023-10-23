use {
    crate::state::{global_state::GlobalState, secure_wrap_token_state::SecureWrapTokenState},
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct CreateSecureWrappedToken<'info> {
    #[account(
        // Only the program authority can create secure wrap tokens.
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>,

    /// CHECK: Separate account for fee-paying, can be same as authority.
    #[account(mut)]
    pub fee_payer: AccountInfo<'info>,

    #[account()]
    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(
        init,
        payer = fee_payer,
        mint::authority = secure_wrap_token_mint,
        mint::freeze_authority = secure_wrap_token_mint,
        mint::decimals = original_token_mint.decimals,
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump,
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = fee_payer,
        seeds = [b"program_original_token_account", original_token_mint.key().as_ref()],
        token::mint = original_token_mint,
        token::authority = program_original_token_account,
        bump
    )]
    pub program_original_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = fee_payer,
        seeds = [b"program_wrapped_token_account", original_token_mint.key().as_ref()],
        token::mint = secure_wrap_token_mint,
        token::authority = program_wrapped_token_account,
        bump,
    )]
    pub program_wrapped_token_account: Box<Account<'info, TokenAccount>>, // Escrow account for Unwrap Discount Auction orders.

    #[account(
        init,
        payer = fee_payer,
        space = 8 + SecureWrapTokenState::INIT_SPACE,
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump,
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    // Programs interacted with.
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

pub fn create_secure_wrapped_token(ctx: Context<CreateSecureWrappedToken>) -> Result<()> {
    let secure_wrap_token_state = &mut ctx.accounts.secure_wrap_token_state;

    secure_wrap_token_state.initialize(
        ctx.accounts.original_token_mint.key(),
        ctx.accounts.secure_wrap_token_mint.key(),
        ctx.bumps.secure_wrap_token_mint,
        ctx.bumps.program_original_token_account,
        ctx.bumps.program_wrapped_token_account,
        ctx.bumps.secure_wrap_token_state,
    );
    Ok(())
}
