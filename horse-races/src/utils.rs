/// Represent `100%` in basis points.
pub const MAX_BPS: u16 = 10_000;

pub fn validate_fee_bps(fee_bps: u16) -> u16 {
    if fee_bps > MAX_BPS {
        panic!("Provided fee bps is greater than max bps!");
    }

    fee_bps
}
