use pinocchio::{AccountView, Address, ProgramResult, cpi::{Seed, Signer}, error::ProgramError};
use pinocchio_system::instructions::Transfer;
use solana_program_log::log;

pub struct Withdraw<'info> {
    owner: &'info AccountView,
    vault: &'info AccountView,
    bump: u8
}

impl<'info> TryFrom<(&'info [AccountView], &'info [u8])> for Withdraw<'info> {
    type Error = ProgramError;

    fn try_from(value: (&'info [AccountView], &'info [u8])) -> Result<Self, Self::Error> {
        let [owner, vault, _] = value.0 else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Checks Owner is a signer
        if !owner.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check vault belongs to System Program
        if !vault.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check vault address matches
        let (vault_address, bump) = Address::find_program_address(&[
            b"vault",
            owner.address().as_ref()
        ], &crate::ID);
        
        if vault_address.ne(vault.address()) {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self { owner, vault, bump })
    }
}

impl<'info> Withdraw<'info> {

    pub fn process(&self) -> ProgramResult {
        log("Withdraw Invoked");

        let bump = [self.bump];

        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.owner.address().as_ref()),
            Seed::from(bump.as_ref())
        ];

        let signers = [
            Signer::from(&seeds)
        ];

        let _ = Transfer {
            from: self.vault,
            to: self.owner,
            lamports: self.vault.lamports()
        }.invoke_signed(&signers);

        Ok(())
    }
}