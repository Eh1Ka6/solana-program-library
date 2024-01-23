use crate::{
    error::TokenError,
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
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    init_data: &InitializeInstructionData,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let mint_account_info = next_account_info(account_info_iter)?;
    let mut mint_data = mint_account_info.data.borrow_mut();
    let mut mint = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut mint_data)?;

    let extension = mint.init_extension::<RebaseMintConfig>(true)?;
    extension.total_supply = init_data.initial_supply;
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

    let mut mint_data = mint_account_info.data.borrow_mut();
    let mut mint = StateWithExtensionsMut::<Mint>::unpack(&mut mint_data)?;
    extension.total_supply = data.new_supply;


    Ok(())
}

pub(crate) fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {

    //check_program_account(program_id)?;
    // here check against our own program account
    match decode_instruction_type(input)? {
        RebaseMintInstruction::Initialize => {
            msg!("RebaseMintInstruction::Initialize");
            let init_data = decode_instruction_data(input)?;
            process_initialize(program_id, accounts, &init_data)
        }
        RebaseMintInstruction::RebaseSupply => {
            msg!("RebaseMintInstruction::RebaseSupply");
            let new_rate = decode_instruction_data(input)?;
            process_rebase_supply(program_id, accounts, new_rate)
        }
    }
}