use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_log::log;

use crate::state::Multisig;

pub fn process_initalize_multisig_instructions(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let [creator, multisig, treasury_wallet, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let seed = [(b"multisig"), creator.key().as_slice()];//slice is type of ref 
    let seeds = &seed[..];
    let (pda_multisig, config_bump) = pubkey::find_program_address(seeds, &crate::ID);
    assert_eq!(&pda_multisig, multisig.key());
    let treasury_seed = [(b"treasury"), multisig.key().as_slice()];
    let treasury_seeds = &treasury_seed[..];
    let (pda_treasury, treasury_bump) = pubkey::find_program_address(treasury_seeds, &crate::ID);
    assert_eq!(&pda_treasury, treasury_wallet.key());

    if multisig.owner() != &crate::ID {
        log!("Creating Multisig Account");

        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: multisig,
            lamports: Rent::get()?.minimum_balance(Multisig::LEN),
            space: Multisig::LEN as u64,
            owner: &crate::ID,
        }
        .invoke()?;

        let multisig_account = Multisig::from_account_info(&multisig)?;
        multisig_account.creator = *creator.key();
        multisig_account.member_count = unsafe { *(data.as_ptr().add(1) as *const u8) };
        multisig_account.member_keys = [Pubkey::default(); 10];
        multisig_account.treasury_wallet = *treasury_wallet.key();
        multisig_account.treasury_bump = treasury_bump;
        multisig_account.config_bump = config_bump;

        match multisig_account.member_count {
            0..=10 => {
                for i in 0..multisig_account.member_count as usize {
                    let member_key = unsafe { *(data.as_ptr().add(2 + i * 32) as *const [u8; 32]) };
                    multisig_account.member_keys[i] = member_key;
                }
            }
            _ => return Err(ProgramError::InvalidAccountData),
        }

        log!("members: {}", unsafe {
            *(data.as_ptr().add(1) as *const u8)
        });
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if treasury_wallet.owner() != &crate::ID {
        log!("Creating Treasury SystemAccount");

        pinocchio_system::instructions::CreateAccount {
            from: creator,
            to: treasury_wallet,
            lamports: Rent::get()?.minimum_balance(0),
            space: 0,
            owner: &pinocchio_system::ID,
        }
        .invoke()?;
    } else {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
