use anchor_lang::prelude::*;

use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate_name: String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

	#[account(
		mut,
		seeds = [POLL_SEED.as_ref() , poll_id.to_le_bytes().as_ref()],
		bump,
	)]
    pub poll_account: Account<'info, PollAccaount>,

	#[account(
		init, 
		payer = signer,
		space = CandidateAccount::DISCRIMINATOR.len() + CandidateAccount::INIT_SPACE,
		seeds = [candidate_name.as_ref(), poll_id.to_le_bytes().as_ref()],
		bump,

	)]
    pub candidate_account: Account<'info, CandidateAccount>,

	pub system_program : Program<'info,System>,

}

impl <'info> InitializeCandidate<'info> {

	pub fn init_canadidate(&mut self, _poll_id: u64, candidate_name: String) ->Result<()>{
		self.candidate_account.candidate_name = candidate_name;
		self.candidate_account.candidate_votes = 0;
		self.poll_account.poll_candidates_count = self.poll_account.poll_candidates_count
        .checked_add(1)
        .unwrap();
		Ok(())
	}	
}
