use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate_name: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
		mut,
		seeds = [POLL_SEED.as_ref() , poll_id.to_le_bytes().as_ref()],
		bump,
	)]
    pub poll_account: Account<'info, PollAccaount>,

    #[account(
	mut,
		seeds = [candidate_name.as_ref(), poll_id.to_le_bytes().as_ref()],
		bump,

	)]
    pub candidate_account: Account<'info, CandidateAccount>,
}

impl<'info> Vote<'info> {
    pub fn vote(&mut self, _poll_id: u64, _candidate_name: String) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;

        if current_time >= self.poll_account.poll_voting_end as i64 {
            return Err(ErrorVote::DonePoll.into());
        } else if current_time < self.poll_account.poll_voting_start as i64 {
            return Err(ErrorVote::UnstaredPoll.into());
        }
        self.candidate_account.candidate_votes = self
            .candidate_account
            .candidate_votes
            .checked_add(1)
            .unwrap();
        Ok(())
    }
}
