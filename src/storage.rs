use anyhow::Result;

trait Storage {
    fn save_challenge(challenge: &str) -> Result<()>;
}
