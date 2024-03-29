use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed, pubkey::Pubkey,
    rent::Rent, system_instruction::create_account, sysvar::Sysvar,
};

#[allow(missing_docs)]
pub struct Cpi {}

impl Cpi {
    #[allow(missing_docs)]
    pub fn create_account<'a>(
        program_id: &Pubkey,
        system_program: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        account_to_create: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        space: usize,
    ) -> ProgramResult {
        let create_state_instruction = create_account(
            fee_payer.key,
            account_to_create.key,
            Rent::get()?.minimum_balance(space),
            space as u64,
            program_id,
        );

        invoke_signed(
            &create_state_instruction,
            &[
                system_program.clone(),
                fee_payer.clone(),
                account_to_create.clone(),
            ],
            &[signer_seeds],
        )
    }
}
