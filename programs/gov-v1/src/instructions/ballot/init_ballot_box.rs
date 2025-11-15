use anchor_lang::prelude::*;

use crate::{error::ErrorCode, BallotBox, ProgramConfig};

#[cfg(not(feature = "skip-pda-check"))]
const GOV_PROGRAM_ID: Pubkey = pubkey!("6MX2RaV2vfTGv6c7zCmRAod2E6MdAgR6be2Vb3NsMxPW");

#[derive(Accounts)]
#[instruction(snapshot_slot: u64, proposal_seed: u64, spl_vote_account: Pubkey)]
pub struct InitBallotBox<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Verifies that signer is a Proposal PDA from the governance program.
    /// When `skip-pda-check` feature is enabled, this check is disabled to allow local testing without CPI.
    /// CHECK: This is verified by the caller.
    pub proposal: UncheckedAccount<'info>,
    #[account(
        init,
        seeds = [
            b"BallotBox".as_ref(),
            &snapshot_slot.to_le_bytes()
        ],
        bump,
        payer = payer,
        space = 8 + BallotBox::INIT_SPACE
    )]
    pub ballot_box: Box<Account<'info, BallotBox>>,
    pub program_config: Box<Account<'info, ProgramConfig>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitBallotBox>,
    snapshot_slot: u64,
    proposal_seed: u64,
    spl_vote_account: Pubkey,
) -> Result<()> {
    let clock = Clock::get()?;

    // Check that snapshot slot is greater than current slot to
    // allow sufficient lead time for snapshot.
    require!(snapshot_slot > clock.slot, ErrorCode::InvalidSnapshotSlot);
    require!(ctx.accounts.proposal.is_signer, ErrorCode::InvalidProposal);
    require!(
        ctx.accounts.proposal.owner == &GOV_PROGRAM_ID,
        ErrorCode::InvalidProposal
    );
    let seeds: &[&[u8]] = &[
        b"proposal".as_ref(),
        &proposal_seed.to_le_bytes(),
        spl_vote_account.as_ref(),
    ];
    let (proposal_pda, _) = Pubkey::find_program_address(seeds, &GOV_PROGRAM_ID);

    require!(
        proposal_pda == ctx.accounts.proposal.key(),
        ErrorCode::InvalidProposal
    );
    let program_config = &ctx.accounts.program_config;
    let ballot_box = &mut ctx.accounts.ballot_box;

    ballot_box.bump = ctx.bumps.ballot_box;
    ballot_box.epoch = clock.epoch;
    ballot_box.slot_created = clock.slot;
    ballot_box.snapshot_slot = snapshot_slot;
    ballot_box.min_consensus_threshold_bps = program_config.min_consensus_threshold_bps;
    ballot_box.vote_expiry_timestamp = clock
        .unix_timestamp
        .checked_add(program_config.vote_duration)
        .unwrap();
    ballot_box.voter_list = program_config.whitelisted_operators.clone();
    ballot_box.tie_breaker_consensus = false;

    Ok(())
}
