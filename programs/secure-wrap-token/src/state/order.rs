use {crate::error::SecureWrapTokenError, anchor_lang::prelude::*};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug, InitSpace)]
pub enum Side {
    None,
    Wrap,
    Unwrap,
}

impl Side {
    const WRAP_SEED: [u8; 1] = [0x01];
    const UNWRAP_SEED: [u8; 1] = [0x02];

    pub fn to_seed(&self) -> Result<[u8; 1]> {
        match self {
            Side::None => err!(SecureWrapTokenError::OrderSideInvalid),
            Side::Wrap => Ok(Self::WRAP_SEED),
            Side::Unwrap => Ok(Self::UNWRAP_SEED),
        }
    }
}

#[account]
#[derive(InitSpace)]
pub struct Order {
    // A token swap order specific to {secure wrap mint, user}.
    pub owner: Pubkey,
    pub secure_wrap_token_mint: Pubkey,
    pub token_account: Pubkey,
    pub secure_wrap_token_account: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub side: Side,
    pub bump: u8,
}

impl Order {
    pub fn initialize(
        &mut self,
        side: Side,
        owner: Pubkey,
        secure_wrap_token_mint: Pubkey,
        token_account: Pubkey,
        secure_wrap_token_account: Pubkey,
        amount_in: u64,
        amount_out: u64,
        bump: u8,
    ) -> Result<()> {
        require!(side != Side::None, SecureWrapTokenError::OrderSideInvalid);
        require_neq!(amount_in, 0, SecureWrapTokenError::InvalidTokenAmount);
        require_neq!(amount_out, 0, SecureWrapTokenError::InvalidTokenAmount);
        if side == Side::Unwrap {
            require_gt!(
                amount_in,
                amount_out,
                SecureWrapTokenError::InvalidOrderAmount,
            );
        } else {
            require_gt!(
                amount_out,
                amount_in,
                SecureWrapTokenError::InvalidOrderAmount,
            );
        }

        self.side = side;
        self.owner = owner;
        self.secure_wrap_token_mint = secure_wrap_token_mint;
        self.token_account = token_account;
        self.secure_wrap_token_account = secure_wrap_token_account;
        self.amount_in = amount_in;
        self.amount_out = amount_out;
        self.bump = bump;
        Ok(())
    }

    pub fn originating_token_account(&self) -> Pubkey {
        if self.side == Side::Unwrap {
            self.secure_wrap_token_account
        } else {
            self.token_account
        }
    }

    pub fn token_account_to_fill(&self) -> Pubkey {
        if self.side == Side::Unwrap {
            self.token_account
        } else {
            self.secure_wrap_token_account
        }
    }
}
