use anchor_lang::prelude::*;

use crate::{error::ErrorCode, BallotBox, ConsensusResult};

#[derive(Accounts)]
pub struct FinalizeBallot<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub ballot_box: Box<Account<'info, BallotBox>>,
    #[account(
        init,
        seeds = [
            b"ConsensusResult".as_ref(),
            &ballot_box.ballot_id.to_le_bytes()
        ],
        bump,
        payer = payer,
        space = 8 + ConsensusResult::INIT_SPACE
    )]
    pub consensus_result: Box<Account<'info, ConsensusResult>>,
    #[account(mut)]
    pub proposal: UncheckedAccount<'info>,
    #[account(address = govcontract::ID)]
    pub govcontract_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FinalizeBallot>) -> Result<()> {
    let ballot_box = &ctx.accounts.ballot_box;
    require!(
        ballot_box.has_consensus_reached(),
        ErrorCode::ConsensusNotReached
    );

    let consensus_result = &mut ctx.accounts.consensus_result;
    consensus_result.ballot_id = ballot_box.ballot_id;
    consensus_result.ballot = ballot_box.winning_ballot.clone();

    // CPI to add merkle tree
    let cpi_accounts = govcontract::cpi::accounts::AddMerkleRoot {
        proposal: ctx.accounts.proposal.to_account_info(),
        consensus_result: ctx.accounts.consensus_result.to_account_info(),
    };
    let signer_seeds = &[&[
        b"ConsensusResult",
        &ballot_box.ballot_id.to_le_bytes(),
        &[ctx.bumps.consensus_result],
    ]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.govcontract_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    govcontract::cpi::add_merkle_root(cpi_ctx)?;

    Ok(())
}
