use crate::{
    error::TokenError,
    check_program_account,
    extension::{
        rebase_mint::{
            instruction::{RebaseMintInstruction, InitializeInstructionData, RebaseSupplyData},
            RebaseMintConfig,
        },
        StateWithExtensionsMut,
    },
    instruction::{decode_instruction_data, decode_instruction_type},
    state::Mint,
    processor::Processor,
};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    supply_authority: &OptionalNonZeroPubkey,
    initial_supply: &u16,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account_info = next_account_info(account_info_iter)?;
    let mut mint_data = mint_account_info.data.borrow_mut();
    let mut mint = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut mint_data)?;

    let extension = mint.init_extension::<RebaseMintConfig>(true)?;
    extension.total_supply = *initial_supply;
    extension.supply_authority = *supply_authority;
    extension.accumulated_rounding_error = 0 as u16;
    Ok(())
}

fn process_rebase_supply(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &RebaseSupplyData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let owner_info_data_len = owner_info.data_len();
    let mut mint_data = mint_account_info.data.borrow_mut();
    let mut mint = StateWithExtensionsMut::<Mint>::unpack(&mut mint_data)?;
    let extension = mint.get_extension_mut::<RebaseMintConfig>()?;
    let supply_authority = Option::<Pubkey>::from(extension.supply_authority).ok_or(TokenError::NoAuthorityExists)?;

   
    Processor::validate_owner(
        program_id,
        &supply_authority,
        owner_info,
        owner_info_data_len,
        account_info_iter.as_slice(),
    )?;
    // Edge case handling: new supply is zero
    if data.new_supply == 0 {
        return Err(TokenError::InvalidSupply.into());
    }
       // Calculate the ratio for adjusting total shares
    let ratio = data.new_supply as f64 / extension.total_supply as f64;
    let new_total_shares = extension.total_shares as f64 * ratio;

    // Adjusting total shares with accumulated rounding error
    let accumulated_error_as_float = extension.accumulated_rounding_error as f64 / 10_000.0;
    let adjusted_total_shares = new_total_shares + accumulated_error_as_float;
    let rounded_total_shares = adjusted_total_shares.round() as u16;

    // Calculate new accumulated rounding error
    let new_error = adjusted_total_shares - rounded_total_shares as f64;
    let new_error_as_u16 = (new_error * 10_000.0).round() as u16;

    // Update the accumulated rounding error and handle distribution
    let potential_new_accumulated_error = extension.accumulated_rounding_error as u32 + new_error_as_u16 as u32;
    
    // Check if accumulated error exceeds the threshold for distributing a share
    if potential_new_accumulated_error >= 10_000 {
        // Distribute one share for every 10,000 units of error
        let shares_to_distribute = potential_new_accumulated_error / 10_000;
        extension.total_shares = extension.total_shares.saturating_add(shares_to_distribute as u16);

        // Adjust the accumulated rounding error
        extension.accumulated_rounding_error = (potential_new_accumulated_error % 10_000) as u16;
    } else {
        extension.accumulated_rounding_error = potential_new_accumulated_error as u16;
    }

    // Update total shares and total supply
    extension.total_shares = rounded_total_shares;
    extension.total_supply = data.new_supply;

    Ok(())
}

pub(crate) fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    check_program_account(program_id)?;
    match decode_instruction_type(input)? {
        RebaseMintInstruction::Initialize => {
            msg!("RebaseMintInstruction::Initialize");
            let InitializeInstructionData {
                supply_authority,
                initial_supply,
            } = decode_instruction_data(input)?;
            process_initialize(program_id, accounts,supply_authority, initial_supply)
        }
        RebaseMintInstruction::RebaseSupply => {
            msg!("RebaseMintInstruction::RebaseSupply");
            let new_supply = decode_instruction_data(input)?;
            process_rebase_supply(program_id, accounts, new_supply)
        }
    }
}