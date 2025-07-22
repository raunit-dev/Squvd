#![allow(unexpected_cfgs)]

#[cfg(feature = "std")]
extern crate std;

use pinocchio::{
    account_info::AccountInfo,
    entrypoint,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
};

mod state;
mod instructions;

use instructions::*;

entrypoint!(process_instruction);


pinocchio_pubkey::declare_id!("H8bpqAoUgRfHh9ViPqH3wjkAVrgGBeg3sA7q5tECz9HC");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = data.split_first().ok_or(ProgramError::InvalidAccountData)?;

    match MultisigInstructions::try_from(discriminator)? {
        MultisigInstructions::InitializeMultisig => instructions::process_initalize_multisig_instructions(accounts, data)?,
        MultisigInstructions::InitializeProposal => instructions::process_initialize_proposal_instruction(accounts, data)?,
        MultisigInstructions::UpdateMultisig => instructions::process_update_multisig_instruction(accounts, data)?,
        MultisigInstructions::Vote => instructions::process_vote_instruction(accounts, data)?,
        MultisigInstructions::CloseProposal => instructions::process_close_proposal_instruction(accounts, data)?,
    }

    Ok(())
}