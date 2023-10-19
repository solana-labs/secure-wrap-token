use {
    crate::{error::SecureWrapTokenError, state::user_token_state::UserTokenState},
    anchor_lang::prelude::*,
    anchor_spl::token::{Burn, FreezeAccount, MintTo, ThawAccount, TokenAccount, Transfer},
};

#[account]
#[derive(InitSpace)]
pub struct SecureWrapTokenState {
    pub original_token_mint: Pubkey,
    pub secure_wrap_token_mint: Pubkey,

    // Invariants:
    // 1) swt_mint.supply == original_token_custody_account.amount + permanently_frozen_token_supply
    // 2) redistributed_token_supply <= permanently_frozen_token_supply
    pub permanently_frozen_token_supply: u64,
    pub redistributed_token_supply: u64,

    // Bumps
    pub secure_wrap_token_mint_bump: u8,
    pub program_original_token_account_bump: u8,
    pub program_wrapped_token_account_bump: u8,
    pub bump: u8,
}

impl SecureWrapTokenState {
    pub fn initialize(
        &mut self,
        original_token_mint: Pubkey,
        secure_wrap_token_mint: Pubkey,
        secure_wrap_token_mint_bump: u8,
        program_original_token_account_bump: u8,
        program_wrapped_token_account_bump: u8,
        bump: u8,
    ) {
        self.original_token_mint = original_token_mint;
        self.secure_wrap_token_mint = secure_wrap_token_mint;
        self.permanently_frozen_token_supply = 0;
        self.redistributed_token_supply = 0;
        self.secure_wrap_token_mint_bump = secure_wrap_token_mint_bump;
        self.program_original_token_account_bump = program_original_token_account_bump;
        self.program_wrapped_token_account_bump = program_wrapped_token_account_bump;
        self.bump = bump;
    }

    pub fn send_original_token<'info>(
        &self,
        token_program: AccountInfo<'info>,
        receiving_account: AccountInfo<'info>,
        program_original_token_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"program_original_token_account",
            &self.original_token_mint.to_bytes(),
            &[self.program_original_token_account_bump],
        ]];
        let transfer_context = CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: program_original_token_account.clone(),
                to: receiving_account,
                authority: program_original_token_account,
            },
            authority_seeds,
        );
        anchor_spl::token::transfer(transfer_context, amount)
    }

    pub fn send_wrapped_token<'info>(
        &self,
        token_program: AccountInfo<'info>,
        receiving_account: AccountInfo<'info>,
        program_wrapped_token_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"program_wrapped_token_account",
            &self.original_token_mint.to_bytes(),
            &[self.program_wrapped_token_account_bump],
        ]];
        let transfer_escrowed_tokens = CpiContext::new_with_signer(
            token_program,
            Transfer {
                from: program_wrapped_token_account.clone(),
                to: receiving_account,
                authority: program_wrapped_token_account,
            },
            authority_seeds,
        );
        anchor_spl::token::transfer(transfer_escrowed_tokens, amount)
    }

    pub fn burn_wrapped_token<'info>(
        &self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        program_wrapped_token_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"program_wrapped_token_account",
            &self.original_token_mint.to_bytes(),
            &[self.program_wrapped_token_account_bump],
        ]];
        let burn_escrowed_tokens = CpiContext::new_with_signer(
            token_program,
            Burn {
                mint: secure_wrap_token_mint,
                from: program_wrapped_token_account.clone(),
                authority: program_wrapped_token_account,
            },
            authority_seeds,
        );
        anchor_spl::token::burn(burn_escrowed_tokens, amount)
    }

    pub fn freeze_account<'info>(
        &self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        account: AccountInfo<'info>,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"secure_wrap_token_mint",
            &self.original_token_mint.to_bytes(),
            &[self.secure_wrap_token_mint_bump],
        ]];
        let freeze_context = CpiContext::new_with_signer(
            token_program,
            FreezeAccount {
                account,
                mint: secure_wrap_token_mint.clone(),
                authority: secure_wrap_token_mint,
            },
            authority_seeds,
        );
        anchor_spl::token::freeze_account(freeze_context)
    }

    fn execute_thaw<'info>(
        &self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        account_to_thaw: AccountInfo<'info>,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"secure_wrap_token_mint",
            &self.original_token_mint.to_bytes(),
            &[self.secure_wrap_token_mint_bump],
        ]];
        let thaw_context = CpiContext::new_with_signer(
            token_program,
            ThawAccount {
                account: account_to_thaw.to_account_info(),
                mint: secure_wrap_token_mint.clone(),
                authority: secure_wrap_token_mint,
            },
            authority_seeds,
        );
        anchor_spl::token::thaw_account(thaw_context)
    }

    pub fn thaw_account<'info>(
        &self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        account_to_thaw: &Account<'info, TokenAccount>,
        user_token_state: &mut UserTokenState,
    ) -> Result<()> {
        user_token_state.validate_thaw(account_to_thaw)?;
        self.execute_thaw(
            token_program,
            secure_wrap_token_mint,
            account_to_thaw.to_account_info(),
        )
    }

    pub fn mint_wrapped_token<'info>(
        &self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        to: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] = &[&[
            b"secure_wrap_token_mint",
            &self.original_token_mint.to_bytes(),
            &[self.secure_wrap_token_mint_bump],
        ]];
        let mint_context = CpiContext::new_with_signer(
            token_program,
            MintTo {
                mint: secure_wrap_token_mint.clone(),
                to,
                authority: secure_wrap_token_mint.clone(),
            },
            authority_seeds,
        );
        anchor_spl::token::mint_to(mint_context, amount)
    }

    pub fn add_permanently_frozen_tokens(&mut self, amount: u64) -> Result<()> {
        self.permanently_frozen_token_supply = self
            .permanently_frozen_token_supply
            .checked_add(amount)
            .ok_or(SecureWrapTokenError::MathOverflow)?;
        Ok(())
    }

    pub fn add_redistributed_tokens(&mut self, amount: u64) -> Result<()> {
        self.redistributed_token_supply = self
            .redistributed_token_supply
            .checked_add(amount)
            .ok_or(SecureWrapTokenError::MathOverflow)?;
        Ok(())
    }

    pub fn send_swt_to_frozen_account<'info>(
        &mut self,
        token_program: AccountInfo<'info>,
        secure_wrap_token_mint: AccountInfo<'info>,
        program_wrapped_token_account: AccountInfo<'info>,
        frozen_account: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        // All in one instruction: unfreeze an frozen account, send SWT to it, and refreeze it.
        self.execute_thaw(
            token_program.clone(),
            secure_wrap_token_mint.clone(),
            frozen_account.clone(),
        )?;
        self.send_wrapped_token(
            token_program.clone(),
            frozen_account.clone(),
            program_wrapped_token_account,
            amount,
        )?;
        self.freeze_account(token_program, secure_wrap_token_mint, frozen_account)?;
        // Record amount of SWT sent to frozen account since it was not recorded on permanent freeze.
        self.add_permanently_frozen_tokens(amount)
    }
}
