use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey}
};
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::clock::Clock;
use pinocchio_log::log;

use crate::state::{Multisig, Proposal, ProposalStatus};

pub fn process_initialize_proposal_instruction(
    accounts: &[AccountInfo],
    _data: &[u8], // Instruction data is not needed for this version
) -> ProgramResult {
    // --- 1. Destructure and Validate Accounts ---
    // Expected accounts:
    // 0: [signer] The member creating the proposal.
    // 1: [writable] The main multisig configuration account.
    // 2: [writable] The new proposal account (PDA) to be created.
    // 3: [] The system program (required by CreateAccount).
    let [creator, multisig_account, proposal_account, system_program, ..] = accounts else {
        log!("Error: Not enough account keys provided.");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // --- 2. Load Multisig State & Authorize Creator ---
    let multisig = Multisig::from_account_info(multisig_account)?;

    // Verify that the creator of the proposal is a valid member of the multisig
    if !multisig.member_keys[..multisig.member_count as usize].contains(creator.key()) {
        log!("Error: Creator is not a member of the multisig.");
        return Err(ProgramError::IllegalOwner);
    }
    
    // Ensure the proposal account is not already initialized
    if proposal_account.owner() == &crate::ID {
        log!("Error: Proposal account is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // --- 3. Derive and Validate Proposal PDA ---
    // Create a unique seed for the new proposal using the multisig key and the current proposal count
    let proposal_seeds = &[
        b"proposal".as_ref(),
        multisig_account.key().as_ref(),
        &multisig.total_proposals.to_le_bytes(),
    ];
    let (pda_proposal, bump_seed) = pubkey::find_program_address(proposal_seeds, &crate::ID);

    // Validate that the provided proposal account key matches our derived PDA
    if &pda_proposal != proposal_account.key() {
        log!("Error: Invalid proposal account PDA.");
        return Err(ProgramError::InvalidArgument);
    }

    // --- 4. Create the Proposal Account via PDA ---
    log!("Creating new proposal account...");

    pinocchio_system::instructions::CreateAccount {
        from: creator,
        to: proposal_account,
        lamports: Rent::get()?.minimum_balance(Proposal::LEN),
        space: Proposal::LEN as u64,
        owner: &crate::ID,
    }
    .invoke()?; // Use invoke_signed for PDA creation

    // --- 5. Initialize Proposal State ---
    log!("Initializing proposal state...");
    let proposal = Proposal::from_account_info(proposal_account)?;
    let clock = Clock::get()?;
    //Storing the Creator also
    proposal.creator = *creator.key();
    // Set the initial state for the new proposal
    proposal.id = multisig.total_proposals;
    proposal.status = ProposalStatus::Active;
    proposal.created_at = clock.unix_timestamp as u64;
    proposal.expiration_time = proposal.created_at + multisig.proposal_expiry;

    // Set all votes to 255 (meaning "Not Voted")
    proposal.votes = [255; 20];

    // Copy voters from the multisig members into the proposal's voter list
    proposal.voter_keys = [Pubkey::default(); 20]; // Reset to default
    let member_count = multisig.member_count as usize;
    proposal.voter_keys[..member_count].copy_from_slice(&multisig.member_keys[..member_count]);


    // --- 6. Update Multisig State ---
    // Increment the total number of proposals in the parent multisig account
    multisig.total_proposals += 1;
    
    log!("Successfully created proposal with ID: {}", proposal.id);

    Ok(())
}