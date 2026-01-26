use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{Escrow, EscrowError};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    // maker：决定条款并将 mint_a 存入 Escrow 的用户
    #[account(mut)]
    pub maker: Signer<'info>,

    // escrow：持有交换条款（创建者、代币铸造、数量）的账户
    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // mint_a：maker 存入的代币
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // mint_b：maker 想要交换的代币
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // maker_ata_a：与 maker 和 mint_a 关联的代币账户，用于将代币存入 vault
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // vault：与 escrow 和 mint_a 关联的代币账户，用于存放存入的代币
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // associated_token_program：用于创建关联代币账户的关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,

    // token_program：用于 CPI 转账的代币程序
    pub token_program: Interface<'info, TokenInterface>,

    // system_program：用于创建 Escrow 的系统程序
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    /// # Create the Escrow
    fn populate_escrow(&mut self, seed: u64, amount: u64, bump: u8) -> Result<()> {
        // 首先使用 set_inner() 辅助工具填充 Escrow
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive: amount,
            bump,
        });
        Ok(())
    }
    /// # Deposit the tokens
    fn deposit_tokens(&self, amount: u64) -> Result<()> {
        // 然后通过 transfer CPI 存入代币
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;
        Ok(())
    }
}

pub fn handler(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
    // Validate the amount
    require_gt!(receive, 0, EscrowError::InvalidAmount);
    require_gt!(amount, 0, EscrowError::InvalidAmount);

    // Save the Escrow Data
    ctx.accounts.populate_escrow(seed, receive, ctx.bumps.escrow)?;

    // Deposit Tokens
    ctx.accounts.deposit_tokens(amount)?;

    Ok(())
}