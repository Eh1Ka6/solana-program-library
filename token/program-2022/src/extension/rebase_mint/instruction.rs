#[cfg(feature = "serde-traits")]
use serde::{Deserialize, Serialize};
use {
    crate::{
        check_program_account,
        instruction::{encode_instruction, TokenInstruction},
    },
    bytemuck::{Pod, Zeroable},
    num_enum::{IntoPrimitive, TryFromPrimitive},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_pod::optional_keys::OptionalNonZeroPubkey,
    std::convert::TryInto,
};
/// Rebase token extension instructions
/// Interesting-bearing mint extension instructions
#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum RebaseMintInstruction {
     /// Initialize a new mint with elastic supply.
    ///
    /// Fails if the mint has already been initialized, so must be called before
    /// `InitializeMint`.
    ///
    /// The mint must have exactly enough space allocated for the base mint (82
    /// bytes), plus 83 bytes of padding, 1 byte reserved for the account type,
    /// then space required for this extension, plus any others.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The mint to initialize.
    ///
    /// Data expected by this instruction:
    ///   `crate::extension::interest_bearing::instruction::InitializeInstructionData`
    Initialize,
    /// Update the total supply. Only supported for mints that include the
    /// `RebaseMintConfig` extension.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single authority
    ///   0. `[writable]` The mint.
    ///   1. `[signer]` The mint supply authority.
    ///
    ///   * Multisignature authority
    ///   0. `[writable]` The mint.
    ///   1. `[]` The mint's multisignature supply authority.
    ///   2. ..2+M `[signer]` M signer accounts.
    ///

    RebaseSupply,
}

/// Data expected by `RebaseMint::Initialize`
#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InitializeInstructionData {
   
    /// The euthorized multisig adresse authorized to rebase the supply.
    pub supply_authority: OptionalNonZeroPubkey,
    /// The initial supply contained inside the pool.
    pub initial_supply: u16,
}

/// Create an `Initialize` instruction
pub fn initialize(
    token_program_id: &Pubkey,
    mint: &Pubkey,
    supply_authority: Option<Pubkey>,
    initial_supply: u16,
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let accounts = vec![AccountMeta::new(*mint, false)];
    Ok(encode_instruction(
        token_program_id,
        accounts,
        TokenInstruction::RebaseMintExtension,
        RebaseMintInstruction::Initialize,
        &InitializeInstructionData {
            // add here optional instruction
            supply_authority: supply_authority.try_into()?,
            initial_supply: initial_supply
        },
    ))
}

/// Data expected by `RebaseMint::RebaseSupply`
#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct RebaseSupplyData {
    /// The new total supply for the token.
    pub new_supply: u16,
}
/// Create an `UpdateSupply` instruction
pub fn update_supply(
    token_program_id: &Pubkey,
    mint: &Pubkey,
    supply_authority: &Pubkey,
    signers: &[&Pubkey],
    new_supply: u16,
    
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;

    let mut accounts = vec![
        AccountMeta::new(*mint, false),
        AccountMeta::new_readonly(*supply_authority, signers.is_empty()),
    ];
    for signer_pubkey in signers.iter() {
        accounts.push(AccountMeta::new_readonly(**signer_pubkey, true));
    }

    let data = RebaseSupplyData { new_supply };

    Ok(encode_instruction(
        token_program_id,
        accounts,
        TokenInstruction::RebaseMintExtension,
        RebaseMintInstruction::RebaseSupply,
        &data,
    ))
}





