use {
    crate::{
        error::SecureWrapTokenError,
        state::{
            global_state::GlobalState,
            order::{Order, Side},
            secure_wrap_token_state::SecureWrapTokenState,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(params: FillOrderProgramParams)]
pub struct FillOrderProgram<'info> {
    #[account(
        constraint = authority.key() == global_state.upgrade_authority
    )]
    pub authority: Signer<'info>,

    pub original_token_mint: Box<Account<'info, Mint>>, // e.g. USDC

    /// CHECK: owner of the swap that the requestor is attempting to fill.
    #[account(mut)]
    pub order_owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"secure_wrap_token_mint", original_token_mint.key().as_ref()],
        bump = secure_wrap_token_state.secure_wrap_token_mint_bump
    )]
    pub secure_wrap_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = order_token_account.mint == original_token_mint.key(),
        constraint = order_token_account.key() == order.token_account,
        constraint = order_token_account.owner == order_owner.key(),
    )]
    pub order_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = order_secure_wrap_token_account.mint == secure_wrap_token_mint.key(),
        constraint = order_secure_wrap_token_account.key() == order.secure_wrap_token_account,
        constraint = order_secure_wrap_token_account.owner == order_owner.key(),
    )]
    pub order_secure_wrap_token_account: Box<Account<'info, TokenAccount>>,

    // Select a wrapped token account to credit with the fees.
    #[account(
        mut,
        constraint = token_account_to_credit.mint == secure_wrap_token_mint.key(),
    )]
    pub token_account_to_credit: Box<Account<'info, TokenAccount>>,

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

    #[account(
        mut,
        seeds = [b"order", original_token_mint.key().as_ref(), order_owner.key().as_ref(), params.side.to_seed()?.as_ref()],
        constraint = order.owner == order_owner.key(),
        constraint = order.side == params.side,
        bump = order.bump,
        close = order_owner,
    )]
    pub order: Box<Account<'info, Order>>,

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
pub struct FillOrderProgramParams {
    side: Side,
}

pub fn fill_order_program(
    ctx: Context<FillOrderProgram>,
    params: &FillOrderProgramParams,
) -> Result<()> {
    ctx.accounts.global_state.validate_order_allowed(
        params.side,
        &ctx.accounts.order_token_account,
        &ctx.accounts.order_secure_wrap_token_account,
    )?;
    require!(
        params.side == Side::Unwrap,
        SecureWrapTokenError::ProgramCannotFillWrapOrders
    );
    // Program authority fills the discounted unwrap order with funds from the custody account.
    let secure_wrap_token_state = &ctx.accounts.secure_wrap_token_state;
    let order = &ctx.accounts.order;

    // Send original tokens from program custody to fill the order.
    secure_wrap_token_state.send_original_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.order_token_account.to_account_info(),
        ctx.accounts
            .program_original_token_account
            .to_account_info(),
        order.amount_out,
    )?;

    // The fill profit is (amount_in - amount_out) and denominated in wrapped token.
    let fill_profit = order
        .amount_in
        .checked_sub(order.amount_out)
        .ok_or(SecureWrapTokenError::MathOverflow)?;
    secure_wrap_token_state.send_wrapped_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.token_account_to_credit.to_account_info(),
        ctx.accounts.program_wrapped_token_account.to_account_info(),
        fill_profit,
    )?;

    // Burn the amount of wrapped tokens whose original tokens have left the protocol.
    secure_wrap_token_state.burn_wrapped_token(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.secure_wrap_token_mint.to_account_info(),
        ctx.accounts.program_wrapped_token_account.to_account_info(),
        order.amount_out,
    )
}
