use {
    crate::state::{
        global_state::GlobalState,
        order::{Order, Side},
        secure_wrap_token_state::SecureWrapTokenState,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(params: CancelOrderParams)]
pub struct CancelOrder<'info> {
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
        constraint = token_account.key() == order.token_account,
        has_one = owner,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = secure_wrap_token_account.mint == secure_wrap_token_mint.key(),
        constraint = secure_wrap_token_account.key() == order.secure_wrap_token_account,
        has_one = owner,
    )]
    pub secure_wrap_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump,
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    #[account(
        mut,
        seeds = [b"order", original_token_mint.key().as_ref(), owner.key().as_ref(), params.side.to_seed()?.as_ref()],
        constraint = order.side == params.side,
        bump = order.bump,
        close = owner,
        has_one = owner,
    )]
    pub order: Box<Account<'info, Order>>,

    #[account(
        mut,
        seeds = [b"program_original_token_account", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.program_original_token_account_bump,
    )]
    pub program_original_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"program_wrapped_token_account", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.program_wrapped_token_account_bump,
    )]
    pub program_wrapped_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [b"global_state"], bump = global_state.bump
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CancelOrderParams {
    side: Side,
}

pub fn cancel_order(ctx: Context<CancelOrder>, params: &CancelOrderParams) -> Result<()> {
    ctx.accounts.global_state.validate_order_allowed(
        params.side,
        &ctx.accounts.token_account,
        &ctx.accounts.secure_wrap_token_account,
    )?;
    let secure_wrap_token_state = &ctx.accounts.secure_wrap_token_state;
    let order = &ctx.accounts.order;
    // Send the escrowed wrapped tokens back to order's original owner.
    if order.side == Side::Unwrap {
        // Transfer wrapped tokens back to the user.
        secure_wrap_token_state.send_wrapped_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.secure_wrap_token_account.to_account_info(),
            ctx.accounts.program_wrapped_token_account.to_account_info(),
            order.amount_in,
        )
    } else {
        // Transfer original tokens back to the user.
        secure_wrap_token_state.send_original_token(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_account.to_account_info(),
            ctx.accounts
                .program_original_token_account
                .to_account_info(),
            order.amount_in,
        )
    }
}
