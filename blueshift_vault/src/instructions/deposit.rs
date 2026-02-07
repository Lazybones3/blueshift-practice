use pinocchio::{AccountView, Address, ProgramResult, error::ProgramError};
use pinocchio_system::instructions::Transfer;
use solana_program_log::log;

pub struct Deposit<'info> {
    owner: &'info AccountView,
    vault: &'info AccountView,
    lamports: u64
}

impl<'info> TryFrom<(&'info [AccountView], &'info [u8])> for Deposit<'info> {
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
        let (vault_address, _) = Address::find_program_address(&[
            b"vault",
            owner.address().as_ref()
        ], &crate::ID);
        
        if vault_address.ne(vault.address()) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check amount is correct length
        if value.1.len() != core::mem::size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let lamports: u64 = u64::from_le_bytes(value.1.try_into().unwrap());
        Ok(Self { owner, vault, lamports })
    }
}

impl<'info> Deposit<'info> {
    pub fn process(&self) -> ProgramResult {
        log("Deposit Invoked");

        let _ = Transfer {
            from: self.owner,
            to: self.vault,
            lamports: self.lamports
        }.invoke();

        Ok(())
    }
}