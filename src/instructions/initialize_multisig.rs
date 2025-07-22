use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::Multisig;

pub fn process_initalize_multisig_instructions(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Destructure expected accounts: [creator, multisig PDA, treasury PDA, ...rest]
    let [creator, multisig, treasury_wallet, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // --- Derive and validate Multisig PDA ---
    let seed = [(b"multisig"), creator.key().as_slice()];
    let seeds = &seed[..];
    let (pda_multisig, config_bump) = pubkey::find_program_address(seeds, &crate::ID);
    assert_eq!(&pda_multisig, multisig.key()); // Ensure passed multisig account is correct PDA

    // --- Derive and validate Treasury PDA ---
    let treasury_seed = [(b"treasury"), multisig.key().as_slice()];
    let treasury_seeds = &treasury_seed[..];
    let (pda_treasury, treasury_bump) = pubkey::find_program_address(treasury_seeds, &crate::ID);
    assert_eq!(&pda_treasury, treasury_wallet.key()); // Ensure passed treasury is correct PDA

    // --- Create and Initialize Multisig Account ---
    if multisig.owner() != &crate::ID {
        log!("Creating Multisig Account");

        // Create the multisig account on-chain with enough rent and space
        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: multisig,
            lamports: Rent::get()?.minimum_balance(Multisig::LEN),
            space: Multisig::LEN as u64,
            owner: &crate::ID,
        }
        .invoke()?;

        // Populate Multisig struct
        let multisig_account = Multisig::from_account_info(&multisig)?;
        multisig_account.creator = *creator.key();

        // Load number of members from data[1]
        multisig_account.member_count = unsafe { *(data.as_ptr().add(1) as *const u8) };

        // Initialize member keys with default, to be overwritten
        multisig_account.member_keys = [Pubkey::default(); 10]; 

        multisig_account.treasury_wallet = *treasury_wallet.key();
        multisig_account.treasury_bump = treasury_bump;
        multisig_account.config_bump = config_bump;

        // Copy member keys from instruction data into the array
        match multisig_account.member_count {
            0..=10 => {
                for i in 0..multisig_account.member_count as usize {
                    let member_key = unsafe { *(data.as_ptr().add(2 + i * 32) as *const [u8; 32]) };
                    multisig_account.member_keys[i] = member_key;
                }
            }
            _ => return Err(ProgramError::InvalidAccountData), // More than 10 members not allowed
        }

        log!("members: {}", unsafe {
            *(data.as_ptr().add(1) as *const u8)
        });
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // --- Create Treasury Account if not already initialized ---
    if treasury_wallet.owner() != &crate::ID {
        log!("Creating Treasury SystemAccount");

        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: treasury_wallet,
            lamports: Rent::get()?.minimum_balance(0), // Just enough rent for a system account
            space: 0,
            owner: &pinocchio_system::ID, // Owned by the system program (not the crate program)
        }
        .invoke()?;
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
