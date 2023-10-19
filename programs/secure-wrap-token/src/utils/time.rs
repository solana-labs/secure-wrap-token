use anchor_lang::{prelude::*, solana_program};

pub fn get_current_time_seconds() -> Result<i64> {
    let time = solana_program::sysvar::clock::Clock::get()?.unix_timestamp;
    if time > 0 {
        Ok(time)
    } else {
        Err(ProgramError::InvalidAccountData.into())
    }
}
