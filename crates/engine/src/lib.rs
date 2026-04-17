pub fn prob_parser(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| "invalid float")?;
    if (0.0..=1.0).contains(&val) {
        Ok(val)
    } else {
        Err("must be between 0 and 1".into())
    }
}

pub fn positive_float_parser(s: &str) -> Result<f64, String> {
    let val: f64 = s.parse().map_err(|_| "invalid float")?;
    if val > 0.0 {
        Ok(val)
    } else {
        Err("must be > 0.0".into())
    }
}

pub mod lob;
