pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("A9W6EiCBJ5qFG4kLAaKeBVLzuvoQDSVV77vNgX6dM7AK");

#[program]
pub mod voting_app {
    use super::*;

    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        poll_id: u64,
        poll_start: u64,
        poll_end: u64,
        poll_name: String,
        poll_description: String,
    ) -> Result<()> {
        ctx.accounts
            .init_poll(poll_id, poll_start, poll_end, poll_name, poll_description)
    }

    pub fn initialize_candidate(
        ctx: Context<InitializeCandidate>,
        poll_id: u64,
        candidate_name: String,
    ) -> Result<()> {
        ctx.accounts.init_canadidate(poll_id, candidate_name)
    }

    pub fn vote(ctx: Context<Vote>, poll_id: u64, candidate_name: String) -> Result<()> {
        ctx.accounts.vote(poll_id, candidate_name)
    }
}
