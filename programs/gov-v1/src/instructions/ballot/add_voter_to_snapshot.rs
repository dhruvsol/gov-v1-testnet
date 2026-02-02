use anchor_lang::prelude::*;

use crate::{error::ErrorCode, BallotBox, ProgramConfig, MAX_OPERATOR_WHITELIST};

/// Permissionless instruction that:
/// - adds `voter` to the `BallotBox.voter_list` snapshot for `snapshot_slot` (if missing)
/// - adds `voter` to `ProgramConfig.whitelisted_operators` (if missing)
///
/// Any signer can call this (intentionally "open"), per request.
#[derive(Accounts)]
#[instruction(snapshot_slot: u64)]
pub struct AddVoterToSnapshot<'info> {
    /// Open signer (no authority gating).
    pub caller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"BallotBox".as_ref(), &snapshot_slot.to_le_bytes()],
        bump = ballot_box.bump
    )]
    pub ballot_box: Box<Account<'info, BallotBox>>,

    #[account(
        mut,
        seeds = [b"ProgramConfig".as_ref()],
        bump
    )]
    pub program_config: Box<Account<'info, ProgramConfig>>,
}

pub fn handler(ctx: Context<AddVoterToSnapshot>, _snapshot_slot: u64, voter: Pubkey) -> Result<()> {
    let ballot_box = &mut ctx.accounts.ballot_box;
    let program_config = &mut ctx.accounts.program_config;

    // Update ballot-box snapshot voter list.
    if !ballot_box.voter_list.contains(&voter) {
        ballot_box.voter_list.push(voter);
        require!(
            ballot_box.voter_list.len() <= MAX_OPERATOR_WHITELIST,
            ErrorCode::VecFull
        );
    }

    // Also update program config whitelist (if missing).
    if !program_config.whitelisted_operators.contains(&voter) {
        program_config.add_operators(Some(vec![voter]))?;
    }

    Ok(())
}

