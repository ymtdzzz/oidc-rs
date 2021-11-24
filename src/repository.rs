use anyhow::Result;

pub trait Repository {
    fn save_challenge(challenge: &str) -> Result<()>;
}
