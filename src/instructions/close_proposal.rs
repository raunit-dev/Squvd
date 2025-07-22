use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::clock::Clock;
use pinocchio_log::log;

use crate::state::{Multisig, Proposal, ProposalStatus};

/// Processes closing or cancelling a proposal.
///
/// This instruction takes a single byte in `data` to determine the action:
/// - `0`: Tally votes for a proposal that has expired or has all votes in.
/// - `1`: Cancel an active proposal. This can only be done by the proposal's creator.
///
/// Accounts expected:
/// 0. `[signer]`   The transaction signer. For cancelling, this must be the proposal creator.
/// 1. `[writable]` The proposal account to be closed/cancelled.
/// 2. `[]`         The parent multisig account (needed for tallying).
pub fn process_close_proposal_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Get the action code (0 for Tally, 1 for Cancel).
    let action = *data.get(0).ok_or(ProgramError::InvalidInstructionData)?;

    // --- Destructure Accounts ---
    let [signer_account, proposal_account, multisig_account, ..] = accounts else {
        log!("Error: Not enough accounts provided. Expected 3.");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !signer_account.is_signer() {
        log!("Error: A signer is required.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut proposal = Proposal::from_account_info(proposal_account)?;

    // --- Action Routing ---
    match action {
        // --- TALLY LOGIC ---
        0 => {
            log!("Action: Tallying proposal ID: {}", proposal.id);
            let multisig = Multisig::from_account_info(multisig_account)?;
            let clock = Clock::get()?;

            if proposal.status != ProposalStatus::Active {
                return Err(ProgramError::InvalidAccountData);
            }

            let is_expired = clock.unix_timestamp as u64 > proposal.expiration_time;
            let eligible_voters = multisig.member_count as usize;
            let votes_cast = proposal.votes[..eligible_voters].iter().filter(|&&v| v != 255).count();
            let all_voted = votes_cast == eligible_voters;

            if !is_expired && !all_voted {
                return Err(ProgramError::InvalidArgument); // Too early to close
            }

            let yes_votes = proposal.votes[..eligible_voters].iter().filter(|&&v| v == 1).count() as u64;
            log!("Yes votes: {} | Required: {}", yes_votes, multisig.threshold);

            if yes_votes >= multisig.threshold {
                proposal.status = ProposalStatus::Succeeded;
                log!("Outcome: Succeeded ✅");
            } else {
                proposal.status = ProposalStatus::Failed;
                log!("Outcome: Failed ❌");
            }
        }
        // --- CANCEL LOGIC ---
        1 => {
            log!("Action: Cancelling proposal ID: {}", proposal.id);
            if proposal.status != ProposalStatus::Active {
                log!("Error: Proposal must be active to be cancelled.");
                return Err(ProgramError::InvalidAccountData);
            }

            // Verify the signer is the original creator of this proposal.
            if proposal.creator != *signer_account.key() {
                log!("Error: Signer is not the creator of the proposal.");
                return Err(ProgramError::IllegalOwner);
            }

            proposal.status = ProposalStatus::Cancelled;
            log!("Outcome: Cancelled 🗑️");
        }
        _ => {
            log!("Error: Invalid action code.");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}