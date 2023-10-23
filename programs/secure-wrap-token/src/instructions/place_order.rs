use {
    crate::{
        state::{
            global_state::GlobalState,
            order::{Order, Side},
            secure_wrap_token_state::SecureWrapTokenState,
        },
        utils::user_utils::transfer_tokens,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(params: PlaceOrderParams)]
pub struct PlaceOrder<'info> {
    // Orders are indefinite until filled or cancelled.
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
        seeds = [b"secure_wrap_token_state", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.bump,
    )]
    pub secure_wrap_token_state: Box<Account<'info, SecureWrapTokenState>>,

    #[account(
        init,
        payer = owner,
        space = 8 + Order::INIT_SPACE,
        seeds = [b"order", original_token_mint.key().as_ref(), owner.key().as_ref(), params.side.to_seed()?.as_ref()],
        bump
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
pub struct PlaceOrderParams {
    side: Side,
    amount_in: u64,
    amount_out: u64,
}

pub fn place_order(ctx: Context<PlaceOrder>, params: &PlaceOrderParams) -> Result<()> {
    let order = &mut ctx.accounts.order;
    order.initialize(
        params.side,
        ctx.accounts.owner.key(),
        ctx.accounts.secure_wrap_token_mint.key(),
        ctx.accounts.token_account.key(),
        ctx.accounts.secure_wrap_token_account.key(),
        params.amount_in,
        params.amount_out,
        ctx.bumps.order,
    )?;
    ctx.accounts.global_state.validate_order_allowed(
        order.side,
        &ctx.accounts.token_account,
        &ctx.accounts.secure_wrap_token_account,
    )?;

    if order.side == Side::Unwrap {
        // Maker transfers wrapped tokens under program custody.
        transfer_tokens(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.secure_wrap_token_account.to_account_info(),
            ctx.accounts.program_wrapped_token_account.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            order.amount_in,
        )
    } else {
        // Maker transfers original tokens under program custody.
        transfer_tokens(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_account.to_account_info(),
            ctx.accounts
                .program_original_token_account
                .to_account_info(),
            ctx.accounts.owner.to_account_info(),
            order.amount_in,
        )
    }
}
