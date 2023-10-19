use {
    crate::{error::SecureWrapTokenError, utils::time::get_current_time_seconds},
    anchor_lang::prelude::*,
    anchor_spl::token::spl_token::state::{Account, AccountState},
};

#[account]
#[derive(InitSpace)]
pub struct PendingUnwrap {
    // A pending unwrap is specific to {mint, user}.
    pub owner: Pubkey,
    pub secure_wrap_token_mint: Pubkey,
    pub secure_wrap_token_account: Pubkey,
    pub token_account: Pubkey,
    pub release_timestamp: i64,
    pub amount: u64,
    pub initialized: bool,
    pub bump: u8,
}

impl PendingUnwrap {
    pub fn initialize(
        &mut self,
        owner: Pubkey,
        secure_wrap_token_mint: Pubkey,
        secure_wrap_token_account: Pubkey,
        token_account: Pubkey,
        release_timestamp: i64,
        amount: u64,
        bump: u8,
    ) {
        self.owner = owner;
        self.secure_wrap_token_mint = secure_wrap_token_mint;
        self.secure_wrap_token_account = secure_wrap_token_account;
        self.token_account = token_account;
        self.release_timestamp = release_timestamp;
        self.amount = amount;
        self.bump = bump;
        self.initialized = true;
    }

    pub fn validate_release(
        &self,
        original_token_account: &Account,
        swt_account: &Account,
    ) -> Result<()> {
        require!(
            original_token_account.state != AccountState::Frozen
                && swt_account.state != AccountState::Frozen,
            SecureWrapTokenError::UnwrapFrozenAccountError
        );

        let current_time = get_current_time_seconds()?;
        require_gte!(
            current_time,
            self.release_timestamp,
            SecureWrapTokenError::PrematurePendingUnwrap
        );
        Ok(())
    }
}
