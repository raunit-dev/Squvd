use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::{Multisig, Proposal, ProposalStatus};

pub fn process_initialize_proposal_instruction(
    accounts: &[AccountInfo],
    _data: &[u8],
) -> ProgramResult {
    let [creator, multisig_account, proposal_account, _system_program, ..] = accounts else {
        log!("Error: Not enough account keys provided.");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let multisig = Multisig::from_account_info(multisig_account)?;

    if !multisig.member_keys[..multisig.member_count as usize].contains(creator.key()) {
        log!("Error: Creator is not a member of the multisig.");
        return Err(ProgramError::IllegalOwner);
    }

    if proposal_account.owner() == &crate::ID {
        log!("Error: Proposal account is already initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let proposal_seeds = &[
        b"proposal".as_ref(),
        multisig_account.key().as_ref(),
        &multisig.total_proposals.to_le_bytes(),
    ];
    let (pda_proposal, _bump_seed) = pubkey::find_program_address(proposal_seeds, &crate::ID);

    if &pda_proposal != proposal_account.key() {
        log!("Error: Invalid proposal account PDA.");
        return Err(ProgramError::InvalidArgument);
    }

    log!("Creating new proposal account...");

    pinocchio_system::instructions::CreateAccount {
        from: creator,
        to: proposal_account,
        lamports: Rent::get()?.minimum_balance(Proposal::LEN),
        space: Proposal::LEN as u64,
        owner: &crate::ID,
    }
    .invoke()?;

    log!("Initializing proposal state...");
    let proposal = Proposal::from_account_info(proposal_account)?;
    let clock = Clock::get()?;
    proposal.creator = *creator.key();
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
    // Increment the total number of proposals in the parent multisig account
    multisig.total_proposals += 1;

    log!("Successfully created proposal with ID: {}", proposal.id);

    Ok(())
}
