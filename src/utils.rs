use anyhow::{Error, Result};

pub fn average(list: &Vec<u16>) -> Result<u16> {
    if list.is_empty() {
        return Err(Error::msg("Cannot calculate average of an empty list"));
    } else {
        let sum: u32 = list.iter().map(|&x| x as u32).sum();
        Ok((sum / list.len() as u32) as u16)
    }
}
