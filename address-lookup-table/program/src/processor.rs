//! Program state processor

use {
    crate::{
        error::{AddressLookupError, ToProgramError},
        instruction::AddressLookupInstruction,
        state::{
            AddressLookupTable, LookupTableMeta, LookupTableStatus, ProgramState,
            LOOKUP_TABLE_MAX_ADDRESSES, LOOKUP_TABLE_META_SIZE,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::{Clock, Slot},
        entrypoint::ProgramResult,
        msg,
        program::invoke,
        program_error::ProgramError,
        program_utils::limited_deserialize,
        pubkey::{Pubkey, PUBKEY_BYTES},
        rent::Rent,
        slot_hashes::SlotHashes,
        system_instruction,
        sysvar::Sysvar,
    },
};

/// Maximum over-the-wire size of a Transaction
///   1280 is IPv6 minimum MTU
///   40 bytes is the size of the IPv6 header
///   8 bytes is the size of the fragment header
pub const PACKET_DATA_SIZE: usize = 1280 - 40 - 8;

fn process_create_lookup_table(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    recent_slot: Slot,
    bump_seed: u8,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let lookup_table_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    // TODO: I _think_ we might need to include the system program in here now
    // let _system_program_info = next_account_info(account_info_iter)?;

    if AddressLookupTable::deserialize(&lookup_table_info.data.borrow()).is_ok() {
        msg!("Table account must not be allocated");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !authority_info.is_signer {
        msg!("Authority account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }
    if !payer_info.is_signer {
        msg!("Payer account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if <SlotHashes as Sysvar>::get()?.get(&recent_slot).is_none() {
        msg!("{} is not a recent slot", recent_slot);
        return Err(ProgramError::InvalidInstructionData);
    }

    let derived_table_key = Pubkey::create_program_address(
        &[
            authority_info.key.as_ref(),
            &recent_slot.to_le_bytes(),
            &[bump_seed],
        ],
        program_id,
    )?;

    if lookup_table_info.key != &derived_table_key {
        msg!(
            "Table address must match derived address: {}",
            derived_table_key
        );
        return Err(ProgramError::InvalidArgument);
    }

    if lookup_table_info.owner == program_id {
        return Ok(());
    }

    let table_account_data_len = LOOKUP_TABLE_META_SIZE;
    let lookup_table_lamports = lookup_table_info.lamports();
    let required_lamports = <Rent as Sysvar>::get()?
        .minimum_balance(table_account_data_len)
        .max(1)
        .saturating_sub(lookup_table_lamports);

    if required_lamports > 0 {
        invoke(
            &system_instruction::transfer(payer_info.key, lookup_table_info.key, required_lamports),
            &[payer_info.clone(), lookup_table_info.clone()],
        )?;
    }

    invoke(
        &system_instruction::allocate(lookup_table_info.key, table_account_data_len as u64),
        &[lookup_table_info.clone()],
    )?;

    invoke(
        &system_instruction::assign(lookup_table_info.key, program_id),
        &[lookup_table_info.clone()],
    )?;

    let lookup_table = ProgramState::LookupTable(LookupTableMeta::new(*authority_info.key));
    let mut data = lookup_table_info.try_borrow_mut_data()?;
    bincode::serialize_into(&mut *data, &lookup_table)
        .map_err::<ProgramError, _>(|_| AddressLookupError::FailedToSerialize.into())?;

    Ok(())
}

fn process_freeze_lookup_table(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let lookup_table_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    if lookup_table_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !authority_info.is_signer {
        msg!("Authority account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let lookup_table_data = lookup_table_info.data.borrow();
    let lookup_table =
        AddressLookupTable::deserialize(&lookup_table_data).map_to_program_error()?;

    if lookup_table.meta.authority.is_none() {
        msg!("Lookup table is already frozen");
        return Err(AddressLookupError::LookupTableImmutable.into());
    }
    if lookup_table.meta.authority != Some(*authority_info.key) {
        return Err(AddressLookupError::IncorrectAuthority.into());
    }
    if lookup_table.meta.deactivation_slot != Slot::MAX {
        msg!("Deactivated tables cannot be frozen");
        return Err(ProgramError::InvalidArgument);
    }
    if lookup_table.addresses.is_empty() {
        msg!("Empty lookup tables cannot be frozen");
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut lookup_table_meta = lookup_table.meta;
    lookup_table_meta.authority = None;
    AddressLookupTable::overwrite_meta_data(
        &mut lookup_table_info.try_borrow_mut_data()?,
        lookup_table_meta,
    )
    .map_to_program_error()?;

    Ok(())
}

fn process_extend_lookup_table(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_addresses: Vec<Pubkey>,
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let lookup_table_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    if lookup_table_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !authority_info.is_signer {
        msg!("Authority account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let lookup_table_data = lookup_table_info.data.borrow();
    let mut lookup_table =
        AddressLookupTable::deserialize(&lookup_table_data).map_to_program_error()?;

    if lookup_table.meta.authority.is_none() {
        return Err(AddressLookupError::LookupTableImmutable.into());
    }
    if lookup_table.meta.authority != Some(*authority_info.key) {
        return Err(AddressLookupError::IncorrectAuthority.into());
    }
    if lookup_table.meta.deactivation_slot != Slot::MAX {
        msg!("Deactivated tables cannot be extended");
        return Err(ProgramError::InvalidArgument);
    }
    if lookup_table.addresses.len() >= LOOKUP_TABLE_MAX_ADDRESSES {
        msg!("Lookup table is full and cannot contain more addresses");
        return Err(ProgramError::InvalidInstructionData);
    }

    if new_addresses.is_empty() {
        msg!("Must extend with at least one address");
        return Err(ProgramError::InvalidInstructionData);
    }

    let new_table_address_len = lookup_table
        .addresses
        .len()
        .saturating_add(new_addresses.len());
    if new_table_address_len > LOOKUP_TABLE_MAX_ADDRESSES {
        msg!(
            "Extended lookup table length {} would exceed max capacity of {}",
            new_table_address_len,
            LOOKUP_TABLE_MAX_ADDRESSES,
        );
        return Err(ProgramError::InvalidInstructionData);
    }

    let clock = <Clock as Sysvar>::get()?;
    if clock.slot != lookup_table.meta.last_extended_slot {
        lookup_table.meta.last_extended_slot = clock.slot;
        lookup_table.meta.last_extended_slot_start_index =
            u8::try_from(lookup_table.addresses.len()).map_err(|_| {
                // This is impossible as long as the length of new_addresses
                // is non-zero and LOOKUP_TABLE_MAX_ADDRESSES == u8::MAX + 1.
                ProgramError::InvalidAccountData
            })?;
    }

    let lookup_table_meta = lookup_table.meta;
    let new_table_data_len = LOOKUP_TABLE_META_SIZE
        .checked_add(new_table_address_len.saturating_mul(PUBKEY_BYTES))
        .ok_or(ProgramError::ArithmeticOverflow)?;
    {
        AddressLookupTable::overwrite_meta_data(
            &mut lookup_table_info.try_borrow_mut_data()?,
            lookup_table_meta,
        )
        .map_to_program_error()?;
    }

    let required_lamports = <Rent as Sysvar>::get()?
        .minimum_balance(new_table_data_len)
        .max(1)
        .saturating_sub(lookup_table_info.lamports());

    if required_lamports > 0 {
        let payer_info = next_account_info(account_info_iter)?;
        if !payer_info.is_signer {
            msg!("Payer account must be a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }
        invoke(
            &system_instruction::transfer(payer_info.key, lookup_table_info.key, required_lamports),
            &[payer_info.clone(), lookup_table_info.clone()],
        )?;
    }

    Ok(())
}

fn process_deactivate_lookup_table(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let lookup_table_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;

    if lookup_table_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !authority_info.is_signer {
        msg!("Authority account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let lookup_table_data = lookup_table_info.data.borrow();
    let lookup_table =
        AddressLookupTable::deserialize(&lookup_table_data).map_to_program_error()?;

    if lookup_table.meta.authority.is_none() {
        msg!("Lookup table is frozen");
        return Err(AddressLookupError::LookupTableImmutable.into());
    }
    if lookup_table.meta.authority != Some(*authority_info.key) {
        return Err(AddressLookupError::IncorrectAuthority.into());
    }
    if lookup_table.meta.deactivation_slot != Slot::MAX {
        msg!("Lookup table is already deactivated");
        return Err(ProgramError::InvalidArgument);
    }

    let mut lookup_table_meta = lookup_table.meta;
    let clock = <Clock as Sysvar>::get()?;
    lookup_table_meta.deactivation_slot = clock.slot;
    AddressLookupTable::overwrite_meta_data(
        &mut lookup_table_info.try_borrow_mut_data()?,
        lookup_table_meta,
    )
    .map_to_program_error()?;

    Ok(())
}

fn process_close_lookup_table(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let lookup_table_info = next_account_info(account_info_iter)?;
    let authority_info = next_account_info(account_info_iter)?;
    let recipient_info = next_account_info(account_info_iter)?;

    if lookup_table_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !authority_info.is_signer {
        msg!("Authority account must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if recipient_info.key == lookup_table_info.key {
        msg!("Lookup table cannot be the recipient of reclaimed lamports");
        return Err(ProgramError::InvalidArgument);
    }

    let lookup_table_data = lookup_table_info.data.borrow();
    let lookup_table =
        AddressLookupTable::deserialize(&lookup_table_data).map_to_program_error()?;

    if lookup_table.meta.authority.is_none() {
        msg!("Lookup table is frozen");
        return Err(AddressLookupError::LookupTableImmutable.into());
    }
    if lookup_table.meta.authority != Some(*authority_info.key) {
        return Err(AddressLookupError::IncorrectAuthority.into());
    }

    let clock = <Clock as Sysvar>::get()?;
    let slot_hashes = <SlotHashes as Sysvar>::get()?;

    match lookup_table.meta.status(clock.slot, &slot_hashes) {
        LookupTableStatus::Activated => {
            msg!("Lookup table is not deactivated");
            Err(ProgramError::InvalidArgument)
        }
        LookupTableStatus::Deactivating { remaining_blocks } => {
            msg!(
                "Table cannot be closed until it's fully deactivated in {} blocks",
                remaining_blocks
            );
            Err(ProgramError::InvalidArgument)
        }
        LookupTableStatus::Deactivated => Ok(()),
    }?;

    let new_recipient_lamports = lookup_table_info
        .lamports()
        .checked_add(recipient_info.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    **lookup_table_info.try_borrow_mut_lamports()? = 0;
    **recipient_info.try_borrow_mut_lamports()? = new_recipient_lamports;

    lookup_table_info.realloc(0, true)?;
    // TODO: Existing program does not reassign
    // lookup_table_info.assign(&system_program::id());

    Ok(())
}

/// Processes an `AddressLookupInstruction`
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction: AddressLookupInstruction =
        limited_deserialize(input, PACKET_DATA_SIZE as u64).map_to_program_error()?;
    match instruction {
        AddressLookupInstruction::CreateLookupTable {
            recent_slot,
            bump_seed,
        } => {
            msg!("Instruction: CreateLookupTable");
            process_create_lookup_table(program_id, accounts, recent_slot, bump_seed)
        }
        AddressLookupInstruction::FreezeLookupTable => {
            msg!("Instruction: FreezeLookupTable");
            process_freeze_lookup_table(program_id, accounts)
        }
        AddressLookupInstruction::ExtendLookupTable { new_addresses } => {
            msg!("Instruction: ExtendLookupTable");
            process_extend_lookup_table(program_id, accounts, new_addresses)
        }
        AddressLookupInstruction::DeactivateLookupTable => {
            msg!("Instruction: DeactivateLookupTable");
            process_deactivate_lookup_table(program_id, accounts)
        }
        AddressLookupInstruction::CloseLookupTable => {
            msg!("Instruction: CloseLookupTable");
            process_close_lookup_table(program_id, accounts)
        }
    }
}
