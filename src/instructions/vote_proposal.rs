use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    ProgramResult,
};
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::clock::Clock;
use pinocchio_log::log;

use crate::state::{Proposal, ProposalStatus, VoteState};

/// Processes a member's vote on an active proposal.
///
/// This instruction validates the voter's eligibility, checks if the proposal is
/// still active, and records the vote. It also updates a personal `VoteState`
/// account for the voter to track their participation.
///
/// Accounts expected:
/// 0. `[signer]`   The member who is casting the vote.
/// 1. `[writable]` The proposal account to vote on.
/// 2. `[writable]` The voter's `VoteState` PDA account.
/// 3. `[]`         The System Program (for potential account creation).
///
/// Instruction data (`data`) expected:
/// - 1 byte: The vote, where `1` = Yes and `0` = No.
pub fn process_vote_instruction(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // --- 1. Destructure Accounts ---
    let [voter_account, proposal_account, vote_state_account, system_program, ..] = accounts else {
        log!("Error: Not enough accounts provided. Expected 4.");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !voter_account.is_signer() {
        log!("Error: The voter account must be a signer.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // --- 2. Load Proposal and Validate Conditions ---
    let proposal = Proposal::from_account_info(proposal_account)?;
    let clock = Clock::get()?;

    // Ensure the proposal is currently active for voting.
    if proposal.status != ProposalStatus::Active {
        log!("Error: This proposal is not currently active.");
        return Err(ProgramError::InvalidAccountData);
    }

    // Ensure the voting period has not expired.
    if clock.unix_timestamp as u64 > proposal.expiration_time {
        log!("Error: Voting has expired for this proposal.");
        // Optionally, update the status to Failed.
        proposal.status = ProposalStatus::Failed;
        return Err(ProgramError::InvalidAccountData);
    }

    // --- 3. Authorize Voter and Record Vote ---
    // Find the voter's position in the list of eligible voters.
    let voter_index = proposal.voter_keys
        .iter()
        .position(|&key| key == *voter_account.key());

    match voter_index {
        Some(index) => {
            // A value of 255 indicates the member has not voted yet.
            if proposal.votes[index] != 255 {
                log!("Error: This member has already voted on this proposal.");
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            let vote = *data.get(0).ok_or(ProgramError::InvalidInstructionData)?;
            if vote > 1 {
                log!("Error: Invalid vote value. Must be 0 (No) or 1 (Yes).");
                return Err(ProgramError::InvalidInstructionData);
            }

            log!("Voter found at index {}. Recording vote: {}.", index, vote);
            proposal.votes[index] = vote;
        }
        None => {
            log!("Error: Signer is not in the list of eligible voters for this proposal.");
            return Err(ProgramError::IllegalOwner);
        }
    }

    // --- 4. Create or Update Voter's Global VoteState PDA ---
    let (pda, bump) = pubkey::find_program_address(&[b"vote_state", voter_account.key().as_ref()], &crate::ID);
    if &pda != vote_state_account.key() {
        log!("Error: Provided VoteState account does not match the derived PDA.");
        return Err(ProgramError::InvalidArgument);
    }

    // If the account owner is the system program, it hasn't been initialized yet.
    if vote_state_account.owner() == system_program.key() {
        log!("First-time voter detected. Creating VoteState account...");

        pinocchio_system::instructions::CreateAccount {
            from: voter_account,
            to: vote_state_account,
            lamports: Rent::get()?.minimum_balance(VoteState::LEN), //
            space: VoteState::LEN as u64, //
            owner: &crate::ID,
        }
        .invoke()?;
        
        // Initialize the new state account.
        let vote_state = VoteState::from_account_info(vote_state_account)?;
        vote_state.is_authorized = true;
        vote_state.total_votes = 1;
        vote_state.config_bump = bump;

    } else {
        // This is an existing voter, so just increment their total vote count.
        log!("Updating existing VoteState for returning voter.");
        let vote_state = VoteState::from_account_info(vote_state_account)?;
        vote_state.total_votes += 1;
    }

    log!(" Vote successfully processed.");
    Ok(())
}