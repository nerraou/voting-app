use anchor_lang::prelude::*;

use crate::constants::*;

use crate::state::PollAccaount;

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init,
		 payer= signer,
		space= PollAccaount::DISCRIMINATOR.len() + PollAccaount::INIT_SPACE,
		seeds = [POLL_SEED.as_ref() , poll_id.to_le_bytes().as_ref()],
		bump,
	)]
    pub poll_account: Account<'info, PollAccaount>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializePoll<'info> {
    pub fn init_poll(
        &mut self,
        _poll_id: u64,
        poll_start: u64,
        poll_end: u64,
        poll_name: String,
        poll_description: String,
    ) -> Result<()> {
        self.poll_account.poll_name = poll_name;
        self.poll_account.poll_description = poll_description;
        self.poll_account.poll_voting_start = poll_start;
        self.poll_account.poll_voting_end = poll_end;
        self.poll_account.poll_candidates_count = 0;

        Ok(())
    }
}
