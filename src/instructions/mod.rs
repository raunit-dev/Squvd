pub mod initialize_multisig;
pub mod initialize_proposal;
pub mod update_multisig;
pub mod vote;
pub mod close_proposal;

pub use initialize_multisig::*;
pub use initialize_proposal::*;
use pinocchio::program_error::ProgramError;
pub use update_multisig::*;
pub use vote::*;
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