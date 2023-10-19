use {
    crate::{error::SecureWrapTokenError, utils::time::get_current_time_seconds},
    anchor_lang::prelude::*,
    anchor_spl::token::spl_token::state::{Account, AccountState},
};

const THREE_DAYS_SECONDS: i64 = 3 * 24 * 60 * 60; // 3 days.
const MAX_FREEZE_PERIOD_SECONDS: u64 = 14 * 24 * 60 * 60; // 14 days.

#[account]
#[derive(InitSpace)]
pub struct UserTokenState {
    // Data specific to {secure wrap token mint, user}.
    secure_wrap_token_account: Pubkey,
    thaw_eligible_timestamp: i64, // i64 is the alias of solana_program::clock::UnixTimestamp.
    thawed_at_timestamp: i64,
    pub permanently_frozen: bool,
    distribute_frozen_funds: u64,

    initialized: bool,
    pub bump: u8,
}

impl UserTokenState {
    pub fn validate_initialization(
        &mut self,
        secure_wrap_token_account: Pubkey,
        bump: u8,
    ) -> Result<()> {
        if self.initialized {
            require_keys_eq!(
                self.secure_wrap_token_account,
                secure_wrap_token_account,
                SecureWrapTokenError::InvalidUserTokenStateAccount,
            );
            require_eq!(
                self.bump,
                bump,
                SecureWrapTokenError::InvalidUserTokenStateAccount,
            );
        } else {
            self.initialized = true;
            self.secure_wrap_token_account = secure_wrap_token_account;
            self.bump = bump;
        }
        Ok(())
    }

    pub fn freeze(&mut self, account: &Account, freeze_period_seconds: u64) -> Result<()> {
        self.validate_freeze_eligibility(account)?;
        self.set_thaw_timestamp(freeze_period_seconds)
    }

    fn validate_freeze_eligibility(&self, account: &Account) -> Result<()> {
        // An account that is currently frozen, or not yet thawed, cannot be frozen again.
        require!(
            account.state != AccountState::Frozen,
            SecureWrapTokenError::IneligibleFreezeError
        );
        // An account is only eligible to be frozen again 3 days after it is thawed.
        // Unwrap() has a max delay of 1 day, so this gives users time to offboard if they wish.
        require_gt!(
            get_current_time_seconds()?,
            self.thawed_at_timestamp + THREE_DAYS_SECONDS,
            SecureWrapTokenError::IneligibleFreezeError
        );
        Ok(())
    }

    fn set_thaw_timestamp(&mut self, freeze_period_seconds: u64) -> Result<()> {
        require_gte!(
            MAX_FREEZE_PERIOD_SECONDS,
            freeze_period_seconds,
            SecureWrapTokenError::FreezePeriodMaximumExceeded
        );
        // Based on current time, set thaw time to be 'freeze_period_seconds' into the future.
        let current_time = get_current_time_seconds()?;
        let freeze_period_seconds: i64 =
            i64::try_from(freeze_period_seconds).map_err(|_| SecureWrapTokenError::MathOverflow)?;
        self.thaw_eligible_timestamp = current_time
            .checked_add(freeze_period_seconds)
            .ok_or(SecureWrapTokenError::MathOverflow)?;
        Ok(())
    }

    pub fn immediate_thaw(&mut self, account: &Account) -> Result<()> {
        require!(
            account.state == AccountState::Frozen,
            SecureWrapTokenError::InvalidThaw
        );
        require!(
            !self.permanently_frozen,
            SecureWrapTokenError::AccountPermanentlyFrozen
        );
        // Proxy the immediate thaw by setting thaw_eligible_timestamp = 0. CPI to SPL-Token needed to complete the thaw.
        self.thaw_eligible_timestamp = 0;
        Ok(())
    }

    pub fn validate_thaw(&mut self, account: &Account) -> Result<()> {
        require!(
            account.state == AccountState::Frozen,
            SecureWrapTokenError::InvalidThaw
        );
        require!(
            !self.permanently_frozen,
            SecureWrapTokenError::AccountPermanentlyFrozen
        );
        require_gte!(
            get_current_time_seconds()?,
            self.thaw_eligible_timestamp,
            SecureWrapTokenError::PrematureThaw
        );
        self.thawed_at_timestamp = get_current_time_seconds()?;
        Ok(())
    }

    pub fn permanent_freeze(
        &mut self,
        original_token_account: &Account,
        swt_account: &Account,
    ) -> Result<()> {
        require!(
            swt_account.state == AccountState::Frozen
                && original_token_account.state == AccountState::Frozen,
            SecureWrapTokenError::InvalidPermanentFreeze
        );
        // After an account is permanently frozen, there is no way to unfreeze.
        self.permanently_frozen = true;
        Ok(())
    }

    pub fn record_frozen_fund_distribution(
        &mut self,
        account: &Account,
        amount: u64,
    ) -> Result<()> {
        require!(
            self.permanently_frozen && account.state == AccountState::Frozen,
            SecureWrapTokenError::InvalidFrozenFundDistribution
        );

        let updated_distribute_frozen_funds =
            self.distribute_frozen_funds.checked_add(amount).unwrap();
        require_gte!(
            account.amount,
            updated_distribute_frozen_funds,
            SecureWrapTokenError::FrozenFundDistributionExceeded,
        );
        self.distribute_frozen_funds = updated_distribute_frozen_funds;
        Ok(())
    }
}
