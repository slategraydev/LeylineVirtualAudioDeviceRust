// Pure Rust Math Logic for WaveRT
// Zero dependencies on wdk-sys

pub struct WaveRTMath;

impl WaveRTMath {
    pub fn calculate_position(
        elapsed_ticks: i64,
        byte_rate: u32,
        frequency: i64,
        buffer_size: usize,
    ) -> u64 {
        if frequency <= 0 {
            return 0;
        }

        let total_bytes = (elapsed_ticks as u128 * byte_rate as u128) / (frequency as u128);
        let mut position = total_bytes as u64;

        if buffer_size > 0 {
            position %= buffer_size as u64;
        }

        position
    }
}
