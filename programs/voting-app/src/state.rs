use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PollAccaount {
    #[max_len(32)]
    pub poll_name: String,

    #[max_len(32)]
    pub poll_description: String,
    pub poll_voting_start: u64,
    pub poll_voting_end: u64,

    pub poll_candidates_count: u64,
}

#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_votes: u64,
}
