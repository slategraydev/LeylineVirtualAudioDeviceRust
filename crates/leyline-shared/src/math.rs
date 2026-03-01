// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// AUDIO MATH
// Precision calculations for WaveRT position and latency.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~



/// Pure Rust Math Logic for WaveRT.
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

    pub fn ticks_to_bytes(elapsed_ticks: i64, byte_rate: u32, frequency: i64) -> u64 {
        if frequency <= 0 {
            return 0;
        }
        ((elapsed_ticks as u128 * byte_rate as u128) / (frequency as u128)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_position_basic() {
        let byte_rate = 48000 * 4;
        let frequency = 10_000_000;
        let elapsed_ticks = 10_000_000;
        let buffer_size = 64 * 1024;

        let pos = WaveRTMath::calculate_position(elapsed_ticks, byte_rate, frequency, buffer_size);
        assert_eq!(pos, 192000 % 65536);
    }

    #[test]
    fn test_calculate_position_half_second() {
        let byte_rate = 44100 * 2;
        let frequency = 10_000_000;
        let elapsed_ticks = 5_000_000;
        let buffer_size = 32 * 1024;

        let pos = WaveRTMath::calculate_position(elapsed_ticks, byte_rate, frequency, buffer_size);
        assert_eq!(pos, 44100 % 32768);
    }
}
