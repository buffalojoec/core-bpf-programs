/// Processes an `SplTokenGroupInstruction`
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = TokenGroupInstruction::unpack(input)?;
    match instruction {
        TokenGroupInstruction::InitializeGroup(data) => {
            msg!("Instruction: InitializeGroup");
            process_initialize_group(program_id, accounts, data)
        }
        TokenGroupInstruction::UpdateGroupMaxSize(data) => {
            msg!("Instruction: UpdateGroupMaxSize");
            process_update_group_max_size(program_id, accounts, data)
        }
        TokenGroupInstruction::UpdateGroupAuthority(data) => {
            msg!("Instruction: UpdateGroupAuthority");
            process_update_group_authority(program_id, accounts, data)
        }
        TokenGroupInstruction::InitializeMember(_) => {
            msg!("Instruction: InitializeMember");
            process_initialize_member(program_id, accounts)
        }
    }
}