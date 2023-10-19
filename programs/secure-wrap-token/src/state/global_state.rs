use {
    crate::{
        error::SecureWrapTokenError, state::order::Side, utils::time::get_current_time_seconds,
    },
    anchor_lang::prelude::*,
    anchor_spl::token::spl_token::state::{Account, AccountState},
};

const MAX_UNWRAP_DELAY_SECONDS: i64 = 86400;

#[account]
#[derive(Default, Debug, InitSpace)]
pub struct GlobalState {
    pub upgrade_authority: Pubkey,
    pub unwrap_delay_seconds: i64,
    pub unwrap_allowed: bool,
    pub orders_allowed: bool,
    pub wrap_orders_allowed: bool,

    pub bump: u8,
}

impl anchor_lang::Id for GlobalState {
    fn id() -> Pubkey {
        crate::ID
    }
}

impl GlobalState {
    pub fn initialize(&mut self, upgrade_authority: Pubkey, bump: u8) {
        self.upgrade_authority = upgrade_authority;
        self.bump = bump;
        // Upon initialization, all order types and unwraps are allowed, unwrap delay time is 24 hours.
        self.unwrap_delay_seconds = MAX_UNWRAP_DELAY_SECONDS;
        self.unwrap_allowed = true;
        self.orders_allowed = true;
        self.wrap_orders_allowed = true;
    }

    pub fn set_unwrap_delay_seconds(&mut self, unwrap_delay_seconds: u64) -> Result<()> {
        let new_unwrap_delay_seconds =
            i64::try_from(unwrap_delay_seconds).map_err(|_| SecureWrapTokenError::MathOverflow)?;
        require_gte!(
            MAX_UNWRAP_DELAY_SECONDS,
            new_unwrap_delay_seconds,
            SecureWrapTokenError::SetUnwrapDelayMaximumExceeded
        );
        self.unwrap_delay_seconds = new_unwrap_delay_seconds;
        Ok(())
    }

    pub fn compute_unwrap_release_timestamp(&self) -> Result<i64> {
        // Based on the current clock time, computes the timestamp at which an unwrap can be released.
        let current_time = get_current_time_seconds()?;
        Ok(current_time.checked_add(self.unwrap_delay_seconds).unwrap())
    }

    pub fn validate_order_allowed(
        &self,
        side: Side,
        order_token_account: &Account,
        order_secure_wrap_token_account: &Account,
    ) -> Result<()> {
        require!(self.orders_allowed, SecureWrapTokenError::OrdersHalted);
        if side == Side::Wrap {
            require!(
                self.wrap_orders_allowed,
                SecureWrapTokenError::WrapOrdersHalted
            );
        }
        require!(
            order_token_account.state != AccountState::Frozen
                && order_secure_wrap_token_account.state != AccountState::Frozen,
            SecureWrapTokenError::FillOrderFrozenAccountError
        );
        Ok(())
    }

    pub fn validate_unwrap_allowed(&self) -> Result<()> {
        require!(self.unwrap_allowed, SecureWrapTokenError::UnwrapHalted);
        Ok(())
    }

    pub fn halt_orders(&mut self) {
        // Halts all order types: wrap, unwrap.
        self.orders_allowed = false;
    }

    pub fn resume_orders(&mut self) {
        self.orders_allowed = true;
    }

    pub fn halt_wrap_orders(&mut self) {
        // Halts only wrap orders.
        self.wrap_orders_allowed = false;
    }

    pub fn resume_wrap_orders(&mut self) {
        self.wrap_orders_allowed = true;
    }

    pub fn halt_unwrap(&mut self) {
        // Halts all time delayed unwraps. Pending unwraps are not allowed to release.
        self.unwrap_allowed = false;
    }

    pub fn resume_unwrap(&mut self) {
        self.unwrap_allowed = true;
    }
}
