use anchor_lang::prelude::*;

declare_id!("GPhW8yzphVJyX7XzUhDaKHbWVSviwvKqoxuWLaiAMkAi");

#[program]
pub mod crypto_donut {
    use super::*;

    pub fn create_wallet(ctx: Context<CreateWallet>) -> ProgramResult {
        let wallet = &mut ctx.accounts.wallet;

        wallet.authority = *ctx.accounts.authority.key;

        Ok(())
    }

    pub fn create_ledger(ctx: Context<CreateLedger>) -> ProgramResult {
        let ledger = &mut ctx.accounts.ledger;

        ledger.authority = ctx.accounts.wallet.authority;

        Ok(())
    }

    pub fn donate(ctx: Context<Donate>, amount: u64) -> ProgramResult {
        let ledger = &mut ctx.accounts.ledger;

        msg!(
            "cap: {}, len: {}",
            ledger.contributors.capacity(),
            ledger.contributors.len()
        );

        if ledger.contributors.len() == 250 {
            return Err(MyError::LedgerOverflow.into());
        }

        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.from.key(),
            &ctx.accounts.to.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.from.to_account_info(),
                ctx.accounts.to.to_account_info(),
            ],
        )?;

        ledger.contributors.push(Contributor {
            authority: *ctx.accounts.from.key,
            amount,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> ProgramResult {
        let wallet = &mut ctx.accounts.wallet.to_account_info();
        let owner = &mut ctx.accounts.authority;

        let current_lamports = owner.lamports();
        **owner.try_borrow_mut_lamports()? =
            current_lamports.checked_add(wallet.lamports()).unwrap();
        **wallet.try_borrow_mut_lamports()? = 0;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateWallet<'info> {
    #[account(
        init,
        payer = authority,
        seeds = [authority.key().as_ref()],
        bump,
        space = 500,
    )]
    pub wallet: Account<'info, Wallet>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateLedger<'info> {
    #[account(
        seeds = [authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub wallet: Account<'info, Wallet>,
    #[account(
        init,
        payer = authority,
        seeds = [wallet.key().as_ref()],
        bump,
        space = 10240,
    )]
    pub ledger: Account<'info, Ledger>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donate<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    #[account(
        mut,
        constraint = to.authority == ledger.authority
    )]
    pub to: Account<'info, Wallet>,
    #[account(
        mut,
        seeds = [to.key().as_ref()],
        bump,
    )]
    pub ledger: Account<'info, Ledger>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [authority.key().as_ref()],
        bump,
        has_one = authority
    )]
    pub wallet: Account<'info, Wallet>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct Wallet {
    authority: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Contributor {
    authority: Pubkey,
    amount: u64,
}

#[account]
pub struct Ledger {
    authority: Pubkey,
    contributors: Vec<Contributor>,
}

#[error]
pub enum MyError {
    #[msg("Ledger overflow, cannot accept new donutes (;")]
    LedgerOverflow,
}
