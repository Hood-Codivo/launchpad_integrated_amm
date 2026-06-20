use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

use crate::state::Pool;

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

pub fn transfer_from_pool<'info>(
    pool: &Account<'info, Pool>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let bump = [pool.bump];
    let signer_seeds: &[&[u8]] = &[b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref(), &bump];

    token::transfer(
        CpiContext::new_with_signer(
            token_program,
            Transfer {
                from,
                to,
                authority: pool.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )
}

pub fn mint_lp_from_pool<'info>(
    pool: &Account<'info, Pool>,
    lp_mint: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let bump = [pool.bump];
    let signer_seeds: &[&[u8]] = &[b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref(), &bump];

    token::mint_to(
        CpiContext::new_with_signer(
            token_program,
            MintTo {
                mint: lp_mint,
                to: destination,
                authority: pool.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )
}
