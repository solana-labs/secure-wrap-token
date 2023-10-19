use anchor_lang::prelude::*;

#[error_code]
pub enum SecureWrapTokenError {
    #[msg("Clock drift error, current time is behind last recorded time")]
    ClockDriftError,
    #[msg("Overflow in arithmetic operation")]
    MathOverflow,
    #[msg("Error deserializing UserTokenState Account")]
    InvalidUserTokenStateAccount,
    #[msg("Thaw requested too early")]
    PrematureThaw,
    #[msg("Permanently frozen account cannot be thawed")]
    AccountPermanentlyFrozen,
    #[msg("Thaw requested on account that is not frozen")]
    InvalidThaw,
    #[msg("Permanent Freeze can only execute when the SWT *and* original token accounts are already frozen")]
    InvalidPermanentFreeze,
    #[msg("Account was recently frozen and is not yet eligible to freeze again")]
    IneligibleFreezeError,
    #[msg("Maximum freeze period is 14 days")]
    FreezePeriodMaximumExceeded,
    #[msg("Redistribute frozen funds can only execute on permanently frozen account")]
    InvalidFrozenFundDistribution,
    #[msg("Cannot distribute funds beyond what is in the permanently frozen account")]
    FrozenFundDistributionExceeded,
    #[msg("Must specify non-zero amount of tokens")]
    InvalidTokenAmount,
    #[msg("Unwrap must have amount_in > amount_out. Wrap must have amount_out > amount_in")]
    InvalidOrderAmount,
    #[msg("Orders are currently halted")]
    OrdersHalted,
    #[msg("Wrap orders are currently halted")]
    WrapOrdersHalted,
    #[msg("PendingUnwrap was not properly initialized")]
    PendingUnwrapNotInitialized,
    #[msg("PendingUnwrap release requested too early")]
    PrematurePendingUnwrap,
    #[msg("Funds cannot be unwrapped from a frozen account")]
    UnwrapFrozenAccountError,
    #[msg("Unwrap is currently halted")]
    UnwrapHalted,
    #[msg("Maximum unwrap delay is 24 hours")]
    SetUnwrapDelayMaximumExceeded,
    #[msg("Order side must be either Unwrap or Wrap")]
    OrderSideInvalid,
    #[msg("Orders belonging to a frozen account cannot be filled")]
    FillOrderFrozenAccountError,
    #[msg("Wrap orders cannot be filled by the program")]
    ProgramCannotFillWrapOrders,
    #[msg("Program can only cancel unwraps for permanently frozen users")]
    ProgramCancelUnwrapError,
    #[msg("Program can only cancel UNWRAP orders for permanently frozen users")]
    ProgramCancelOrderError,
}
