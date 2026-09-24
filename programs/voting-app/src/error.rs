use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorVote {
    #[msg("The poll did not start yet")]
    UnstaredPoll,
    #[msg("The poll is done")]
    DonePoll,
}
