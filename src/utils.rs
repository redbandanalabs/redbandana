use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    rent::Rent,
    sysvar::Sysvar,
};

/// Ensure an account is rent exempt.
pub fn assert_rent_exempt(
    account_info: &AccountInfo,
    data_len: usize,
) -> Result<(), ProgramError> {
    let rent = Rent::get()?;
    if !rent.is_exempt(account_info.lamports(), data_len) {
        return Err(ProgramError::AccountNotRentExempt);
    }
    Ok(())
}
