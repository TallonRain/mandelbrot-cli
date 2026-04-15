//! ASCII density ramp: maps a normalized escape value to a printable glyph.

pub const RAMP: &[u8] = b" .:-=+*#%@";

/// Map a normalized value in `[0.0, 1.0]` to a character in the density ramp.
/// Values outside the range are clamped. Inside-set points (t = 1.0) produce
/// the densest glyph.
pub fn density_char(t: f64) -> char {
    let clamped = t.clamp(0.0, 1.0);
    let idx = (clamped * (RAMP.len() - 1) as f64).round() as usize;
    RAMP[idx] as char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_values_map_to_ramp_endpoints() {
        assert_eq!(density_char(0.0), ' ');
        assert_eq!(density_char(1.0), '@');
    }

    #[test]
    fn out_of_range_is_clamped() {
        assert_eq!(density_char(-5.0), ' ');
        assert_eq!(density_char(5.0), '@');
    }

    #[test]
    fn midpoint_lands_near_ramp_middle() {
        let mid = density_char(0.5);
        let mid_idx = RAMP.len() / 2;
        // Allow either of the two middle glyphs depending on rounding.
        assert!(
            mid == RAMP[mid_idx] as char || mid == RAMP[mid_idx - 1] as char,
            "unexpected midpoint glyph: {mid}"
        );
    }
}
