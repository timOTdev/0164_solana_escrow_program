use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
};

use crate::{instruction::EscrowInstruction, error::EscrowError};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        // Takes the instruction_data from entrypoint and "unpacks" it.
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        // Then we can figure how to handle it and which processing function to call.
        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                // Just a logging message to let us know where we are in process.
                msg!("Instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }
        }
    }

    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // Make the accounts iterable before we can do anything.
        let account_info_iter = &mut accounts.iter();

        // We need to check that the first account is the person that initialized the escrow account.
        let initializer = next_account_info(account_info_iter)?;

        // If the initilizer person is not the same as the signer, then error out.
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Pull out the temporary token account from iteration.
        // By default, the token account is the owner of this temp token account.
        // Later the ownership will transfer from the token accoutn to the PDA.
        let temp_token_account = next_account_info(account_info_iter)?;

        // No changes here. This is sent to Bob's account so the escrow knows later where to send Asset Y.
        let token_to_receive_account = next_account_info(account_info_iter)?;

        // This serves as a check for when the Bob makes the transaction.
        // Basically, the owner of of this token to receive account should be the escrow account later.
        // Remember, that only the owners can make changes to the account.
        // So when escrow account takes ownership of Bob's Asset Y account later,
        // It can designate that account and set it to Alice's Asset Y account.
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }
}
