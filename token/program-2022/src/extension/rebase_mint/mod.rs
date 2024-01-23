#[cfg(feature = "serde-traits")]
use serde::{Deserialize, Serialize};
use {
    crate::extension::{Extension, ExtensionType},
    bytemuck::{Pod, Zeroable},
    solana_program::program_error::ProgramError,
   
    spl_pod::{
        optional_keys::OptionalNonZeroPubkey,
    },
};

/// Rebasing token extension instructions
pub mod instruction;

/// Rebasing token extension processor
pub mod processor;

/// Rebasing token extension data for mints
#[repr(C)]
#[cfg_attr(feature = "serde-traits", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-traits", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct RebaseMintConfig {
    /// Total supply of the token
    pub total_supply: u64,
    /// Total shares of the token
    pub total_shares: u64,
    /// Authority that can set the supply and authority
    pub supply_authority: OptionalNonZeroPubkey,
}

impl RebaseMintConfig {
    //// Convert a token amount to its equivalent in shares.
    /// 
    /// # Arguments
    /// * `amount` - The amount of tokens to convert to shares.
    ///
    /// # Returns
    /// The equivalent number of shares for the given token amount.
    fn amount_to_shares(&self, amount: u64) -> u64 {
        if self.total_supply == 0 {
            // Edge case: If total supply is zero, treat the conversion ratio as 1:1
            amount
        } else {
            // Calculate the share-to-token ratio and convert the token amount to shares
            let ratio = self.total_shares as f64 / self.total_supply as f64;
            (amount as f64 * ratio).round() as u64
        }
    }

    /// Convert shares to token amount based on the current share-to-token ratio.
    /// 
    /// # Arguments
    /// * `shares` - The number of shares to convert to tokens.
    ///
    /// # Returns
    /// The equivalent token amount for the given number of shares.
   fn shares_to_amount(&self, shares: u64) -> u64 {
        if self.total_shares == 0 {
            // Edge case: If total shares is zero, treat the conversion ratio as 1:1
            shares
        } else {
            // Calculate the token-to-share ratio and convert the shares to token amount
            let ratio = self.total_supply as f64 / self.total_shares as f64;
            (shares as f64 * ratio).round() as u64
        }
    }

    /// Convert shares to UI amount representation.
    ///
    /// # Arguments
    /// * `shares` - The number of shares to convert.
    /// * `decimals` - The number of decimals used by the token.
    ///
    /// # Returns
    /// The UI representation of the token amount equivalent to the given shares.
    pub fn shares_to_ui_amount(&self, shares: u64, decimals: u8) -> Option<String> {
        // Convert shares to the raw token amount
        let amount = self.shares_to_amount(shares);

        // Adjust the amount for token decimals and format it as a string
        let ui_amount = amount as f64 / 10_f64.powi(decimals as i32);
        Some(format!("{:.*}", decimals as usize, ui_amount))
    }

      /// Try to convert a UI representation of a token amount to its equivalent number of shares.
    ///
    /// # Arguments
    /// * `ui_amount` - The UI representation of the token amount.
    /// * `decimals` - The number of decimals used by the token.
    ///
    /// # Returns
    /// The equivalent number of shares for the given UI token amount.
    pub fn try_ui_amount_into_shares(&self, ui_amount: &str, decimals: u8) -> Result<u64, ProgramError> {
        let scaled_amount = ui_amount
            .parse::<f64>()
            .map_err(|_| ProgramError::InvalidArgument)?;

        // Adjust for token decimals
        let amount = scaled_amount * 10_f64.powi(decimals as i32);

        if amount > (u64::MAX as f64) || amount < 0.0 || amount.is_nan() {
            Err(ProgramError::InvalidArgument)
        } else {
            // Convert the adjusted token amount to shares
            Ok(self.amount_to_shares(amount as u64))
        }
    }
}

impl Extension for RebaseMintConfig {
    const TYPE: ExtensionType = ExtensionType::RebaseMintConfig;
    // Additional implementation details for the extension
}

