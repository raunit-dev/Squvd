use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
#[derive(PartialEq)]
pub struct Proposal {
    pub creator: Pubkey,              // Address that created the proposal
    pub id: u64,                  // Unique ID of the proposal
    pub expiration_time: u64,     // When the proposal expires
    pub status: ProposalStatus,   // Current state of the proposal
    pub voter_keys: [Pubkey; 20], // Eligible voter public keys
    pub votes: [u8; 20],          // 1 (yes), 0 (no), or 255 (not voted) // 255 is the maximum value for a u8 often used as "not set" value
    pub created_at: u64,          // Timestamp of proposal creation
}

impl Proposal {
    pub const LEN: usize = 8 + 8 + 1 + (32 * 20) + 20 + 8 + 32;

    pub fn from_account_info_unchecked(account_info: &AccountInfo) -> &mut Self {
        unsafe { &mut *(account_info.borrow_mut_data_unchecked().as_ptr() as *mut Self) }
    }

    pub fn from_account_info(
        account_info: &AccountInfo,
    ) -> Result<&mut Self, pinocchio::program_error::ProgramError> {
        if account_info.data_len() < Self::LEN {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        Ok(Self::from_account_info_unchecked(account_info))
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum ProposalStatus {
    Draft = 0,     // Not Actually using it (will use it V2)
    Active = 1,    // Currently open for voting
    Failed = 2,    // Did not meet threshold or expired
    Succeeded = 3, // Met threshold and succeeded
    Cancelled = 4, // Manually cancelled before conclusion
}

impl TryFrom<&u8> for ProposalStatus {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(ProposalStatus::Draft),
            1 => Ok(ProposalStatus::Active),
            2 => Ok(ProposalStatus::Failed),
            3 => Ok(ProposalStatus::Succeeded),
            4 => Ok(ProposalStatus::Cancelled),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
