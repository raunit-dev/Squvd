use pinocchio::program_error::ProgramError;

pub mod initialize_multisig;
pub mod initialize_proposal;
pub mod update_multisig;
pub mod close_proposal;
pub mod vote_proposal;

pub use initialize_multisig::*;
pub use initialize_proposal::*;
pub use update_multisig::*;
pub use vote_proposal::*;
pub use close_proposal::*;

pub enum MultisigInstructions {
    InitializeMultisig = 0,
    InitializeProposal = 1,
    UpdateMultisig = 2,
    Vote = 3,
    CloseProposal = 4,
}


impl TryFrom<&u8> for MultisigInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MultisigInstructions::InitializeMultisig),
            1 => Ok(MultisigInstructions::InitializeProposal),
            2 => Ok(MultisigInstructions::UpdateMultisig),
            3 => Ok(MultisigInstructions::Vote),
            4 => Ok(MultisigInstructions::CloseProposal),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}