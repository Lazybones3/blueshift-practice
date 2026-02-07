use pinocchio::{AccountView, Address, ProgramResult, cpi::{Seed, Signer}, error::ProgramError};
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::TokenAccount};

use crate::{Escrow, helpers::{AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, MintInterface, ProgramAccount, SignerAccount}};


pub struct TakeAccounts<'a> {
    pub taker: &'a AccountView,
    pub maker: &'a AccountView,
    pub escrow: &'a AccountView,
    pub mint_a: &'a AccountView,
    pub mint_b: &'a AccountView,
    pub vault: &'a AccountView,
    pub taker_ata_a: &'a AccountView,
    pub taker_ata_b: &'a AccountView,
    pub maker_ata_b: &'a AccountView,
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 11 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let taker = &accounts[0];
        let maker = &accounts[1];
        let escrow = &accounts[2];
        let mint_a = &accounts[3];
        let mint_b = &accounts[4];
        let vault = &accounts[5];
        let taker_ata_a = &accounts[6];
        let taker_ata_b = &accounts[7];
        let maker_ata_b = &accounts[8];
        let system_program = &accounts[9];
        let token_program = &accounts[10];

        // 基本账户检查
        SignerAccount::check(taker)?;
        ProgramAccount::check(escrow)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b, token_program)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        Ok(Self { taker, maker, escrow, mint_a, mint_b, vault, taker_ata_a, taker_ata_b, maker_ata_b, system_program, token_program })
    }
}

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>
}

impl<'a> TryFrom<&'a [AccountView]> for Take<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(accounts)?;

        // 初始化必要的 ATA（如果不存在）
        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,
            accounts.mint_a,
            accounts.taker,
            accounts.taker,
            accounts.system_program,
            accounts.token_program,
        )?;

        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,
            accounts.mint_b,
            accounts.taker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow()?;
        let escrow = Escrow::load(&data)?;

        let escrow_key = Address::create_program_address(
            &[
                b"escrow",
                self.accounts.maker.address().as_ref(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ], &crate::ID
        )?;

        if &escrow_key != self.accounts.escrow.address() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];

        let signer = Signer::from(&escrow_seeds);

        let amount = TokenAccount::from_account_view(self.accounts.vault)?.amount();

        let receive_amount = escrow.receive;

        // 1. 从 Vault 转移到 Taker (使用 PDA 签名)
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.taker_ata_a,
            authority: self.accounts.escrow,
            amount,
        }.invoke_signed(&[signer.clone()])?;

        // 2. 关闭 Vault (使用 PDA 签名)
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }.invoke_signed(&[signer.clone()])?;

        // 3. 从 Taker 转移到 Maker
        Transfer {
            from: self.accounts.taker_ata_b,
            to: self.accounts.maker_ata_b,
            authority: self.accounts.taker,
            amount: receive_amount,
        }.invoke()?;

        // 4. 关闭 Escrow（转移 lamports 到 maker 并清零数据）
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

        drop(data);

        Ok(())
    } 
}
