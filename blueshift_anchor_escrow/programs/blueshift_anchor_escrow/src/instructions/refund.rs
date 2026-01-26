use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{Escrow, EscrowError};

#[derive(Accounts)]
pub struct Refund<'info> {
    // maker：决定交换条款的用户
    #[account(mut)]
    pub maker: Signer<'info>,

    // escrow：存储所有交换条款的账户
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
    )]
    pub escrow: Box<Account<'info, Escrow>>,

    // mint_a：maker 存入的代币
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    // vault：与 escrow 和 mint_a 关联的代币账户，代币已存入其中
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // maker_ata_a：与 maker 和 mint_a 关联的代币账户，将从 vault 接收代币
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // associated_token_program：用于创建关联代币账户的关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
    // token_program：用于 CPI 转账的代币程序
    pub token_program: Interface<'info, TokenInterface>,
    // system_program：用于创建 Escrow 的系统程序
    pub system_program: Program<'info, System>,
}

impl<'info> Refund<'info> {
    fn refund_and_close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // 1.将代币从 vault 转移到 maker_ata_a
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        // 3.然后关闭现在已空的金库
        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.vault.to_account_info(),
                authority: self.escrow.to_account_info(),
                destination: self.maker.to_account_info(),
            },
            &signer_seeds,
        ))?;

        Ok(())
    }
}

pub fn handler(ctx: Context<Refund>) -> Result<()> {
    ctx.accounts.refund_and_close_vault()?;
    Ok(())
}
