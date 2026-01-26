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
pub struct Take<'info> {
    // taker：接受 maker 条款并进行交换的用户
    #[account(mut)]
    pub taker: Signer<'info>,

    // maker：最初设定条款的用户
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    // escrow：存储此交换所有条款的账户
    #[account(
        mut,
        close = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        has_one = maker @ EscrowError::InvalidMaker,
        has_one = mint_a @ EscrowError::InvalidMintA,
        has_one = mint_b @ EscrowError::InvalidMintB,
    )]
    pub escrow: Box<Account<'info, Escrow>>,

    /// Token Accounts
    // mint_a：maker 存入的代币
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    // mint_b：maker 希望交换的代币
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    // vault：与 escrow 和 mint_a 关联的代币账户，将代币发送给 taker
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    // taker_ata_a：与 taker 和 mint_a 关联的代币账户，将从 vault 接收代币
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    // taker_ata_b：与 taker 和 mint_b 关联的代币账户，将代币发送给 maker
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    // maker_ata_b：与 maker 和 mint_b 关联的代币账户，将接收来自 taker 的代币
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Programs
    // associated_token_program：用于创建关联代币账户的关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
    // token_program：用于 CPI 转账的代币程序
    pub token_program: Interface<'info, TokenInterface>,
    // system_program：用于创建 Escrow 的系统程序
    pub system_program: Program<'info, System>,
}

impl<'info> Take<'info> {
    fn transfer_to_maker(&mut self) -> Result<()> {
        // 1.首先将代币从taker_ata_b转移到maker_ata_b
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.taker_ata_b.to_account_info(),
                    to: self.maker_ata_b.to_account_info(),
                    mint: self.mint_b.to_account_info(),
                    authority: self.taker.to_account_info(),
                },
            ),
            self.escrow.receive,
            self.mint_b.decimals,
        )?;

        Ok(())
    }

    fn withdraw_and_close_vault(&mut self) -> Result<()> {
        // Create the signer seeds for the Vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // 2.然后将代币从vault转移到taker_ata_a
        // Transfer Token A (Vault -> Taker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.taker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    authority: self.escrow.to_account_info(),
                },
                &signer_seeds,
            ),
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        // 3.最后关闭现在已空的保险库
        // Close the Vault
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

pub fn handler(ctx: Context<Take>) -> Result<()> {
    // Transfer Token B to Maker
    ctx.accounts.transfer_to_maker()?;

    // Withdraw and close the Vault
    ctx.accounts.withdraw_and_close_vault()?;

    Ok(())
}