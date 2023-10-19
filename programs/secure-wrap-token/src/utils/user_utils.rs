use {anchor_lang::prelude::*, anchor_spl::token::Transfer};

pub fn transfer_tokens<'info>(
    token_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let transfer_context = CpiContext::new(
        token_program,
        Transfer {
            from,
            to,
            authority,
        },
    );
    anchor_spl::token::transfer(transfer_context, amount)
}
