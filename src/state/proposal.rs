use pinocchio::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[repr(C)]
pub struct Proposal {
    pub id: u64,                         // Unique ID of the proposal
    pub expiration_time: u64,           // When the proposal expires
    pub status: ProposalStatus,         // Current state of the proposal
    pub voter_keys: [Pubkey; 20],       // Eligible voter public keys
    pub votes: [u8; 20],                // 1 (yes), 0 (no), or 255 (not voted)
    pub created_at: u64,                // Timestamp of proposal creation
}

impl Proposal {
    pub const LEN: usize = 8 + 8 + 1 + (32 * 20) + 20 + 8;
}

#[repr(u8)]
pub enum ProposalStatus {
    Draft = 0,       // Proposal created but not yet active
    Active = 1,      // Currently open for voting
    Failed = 2,      // Did not meet threshold or expired
    Succeeded = 3,   // Met threshold and succeeded
    Cancelled = 4,   // Manually cancelled before conclusion
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
