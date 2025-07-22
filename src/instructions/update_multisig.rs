use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::Multisig;

/// Processes the instruction to update core parameters of a multisig account.
///
/// This action can only be performed by the original creator of the multisig,
/// who must sign the transaction.
///
/// Accounts expected:
/// 0. `[signer]` The creator of the multisig.
/// 1. `[writable]` The multisig account to be updated.
///
/// Instruction data (`data`) expected:
/// - First 8 bytes: The new voting threshold (`u64`).
/// - Next 8 bytes: The new proposal expiry duration in seconds (`u64`).
pub fn process_update_multisig_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // --- 1. Destructure Accounts ---
    let [creator_account, multisig_account, ..] = accounts else {
        log!("Error: Not enough accounts provided. Expected 2.");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !creator_account.is_signer() {
        log!("Error: The creator account must sign the transaction.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // --- 2. Load Multisig State ---
    let mut multisig = Multisig::from_account_info(multisig_account)?;

    // --- 3. Authorize Signer ---
    // Verify that the signer is the original creator stored in the multisig state.
    if multisig.creator != *creator_account.key() {
        log!("Error: Signer is not the authorized creator of this multisig.");
        return Err(ProgramError::IllegalOwner);
    }

    // --- 4. Process Instruction Data ---
    if data.len() < 16 {
        log!("Error: Instruction data is invalid. Expected 16 bytes.");
        return Err(ProgramError::InvalidInstructionData);
    }
    
    // Safely deserialize the data into new values.
    let new_threshold = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let new_proposal_expiry = u64::from_le_bytes(data[8..16].try_into().unwrap());

    // --- 5. Validate and Update ---
    log!("Current threshold: {}, New threshold: {}", multisig.threshold, new_threshold);
    log!("Current expiry: {}, New expiry: {}", multisig.proposal_expiry, new_proposal_expiry);

    // The new threshold must be greater than 0 and cannot exceed the number of members.
    if new_threshold == 0 || new_threshold > multisig.member_count as u64 {
        log!("Error: Invalid threshold. It must be between 1 and the member count.");
        return Err(ProgramError::InvalidInstructionData);
    }

    multisig.threshold = new_threshold;
    multisig.proposal_expiry = new_proposal_expiry;

    log!("Multisig successfully updated.");

    Ok(())
}