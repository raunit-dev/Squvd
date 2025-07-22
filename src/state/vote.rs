use pinocchio::{
    account_info::AccountInfo,
    pubkey::Pubkey,
};

#[repr(C)]
pub struct VoteState {
    pub is_authorized: bool,   // Whether the user is allowed to vote
    pub total_votes: u64,      // Number of times this voter has voted
    pub config_bump: u8,       // PDA bump for this VoteState account
}

impl VoteState {
    pub const LEN: usize = 1 + 8 + 1;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> &mut Self {
        unsafe { &mut *(account_info.borrow_data_unchecked().as_ptr() as *mut Self) }
    }

    pub fn from_account_info(account_info: &AccountInfo) -> Result<&mut Self, pinocchio::program_error::ProgramError> {
        if account_info.data_len() < Self::LEN {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }

        Ok(Self::from_account_info_unchecked(account_info))
    }
}
