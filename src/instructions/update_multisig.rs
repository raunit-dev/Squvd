use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};
use pinocchio_log::log;

use crate::state::Multisig;

//This action can only be performed by the original creator of the multisig
//who must sign the transaction
// Instruction data (data) expected
//First 8 bytes: The new voting threshold (u64)
//Next 8 bytes: The new proposal expiry duration in seconds (u64)
//Tried to keep the update_multisig as simple as possible for now
pub fn process_update_multisig_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let [creator_account, multisig_account, ..] = accounts else {
        log!("Error: Not enough accounts provided. Expected 2.");//Validating the accounts 
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator_account.is_signer() { //The one who created The Multi is the one who can sign these instruction (Update_multisig)
        log!("Error: The creator account must sign the transaction.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let multisig = Multisig::from_account_info(multisig_account)?;

    // Verify that the signer is the original creator stored in the multisig state.
    if multisig.creator != *creator_account.key() { // you cannot compare a [u8;32] with &[u8;32]
        log!("Error: Signer is not the authorized creator of this multisig.");
        return Err(ProgramError::IllegalOwner);
    }

    if data.len() < 16 { //Good way to make sure the user is passing the instruction
        log!("Error: Instruction data is invalid. Expected 16 bytes.");
        return Err(ProgramError::InvalidInstructionData);
    }

    let new_threshold = u64::from_le_bytes(data[0..8].try_into().unwrap());//getting the new threshold
    let new_proposal_expiry = u64::from_le_bytes(data[8..16].try_into().unwrap());//getting the new expiry

    //Validate and Update
    log!(
        "Current threshold: {}, New threshold: {}",
        multisig.threshold,
        new_threshold
    );
    log!(
        "Current expiry: {}, New expiry: {}",
        multisig.proposal_expiry,
        new_proposal_expiry
    );

    // The new threshold must be greater than 0 and cannot exceed the number of members
    if new_threshold == 0 || new_threshold > multisig.member_count as u64 {
        log!("Error: Invalid threshold. It must be between 1 and the member count.");
        return Err(ProgramError::InvalidInstructionData);
    }

    multisig.threshold = new_threshold;
    multisig.proposal_expiry = new_proposal_expiry;

    log!("Multisig successfully updated.");

    Ok(())
}
