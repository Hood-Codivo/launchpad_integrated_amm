use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::BondingCurve;

pub fn transfer_tokens<'info>(
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    token::transfer(
        CpiContext::new(
            token_program,
            Transfer { from, to, authority },
        ),
        amount,
    )
}

pub fn transfer_from_curve<'info>(
    curve: &Account<'info, BondingCurve>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let bump = [curve.bump];
    let signer_seeds: &[&[u8]] = &[b"curve", curve.mint.as_ref(), &bump];

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from,
                to,
                authority: curve.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )
}
