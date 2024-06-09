use core::fmt;

#[cfg(feature = "debug_cycle_counts")]
use core::arch::x86_64::_rdtsc;

use num_format::{Locale, ToFormattedString};

#[derive(Default, Debug, Copy, Clone)]
pub struct CycleCounter {
    pub last_start_cycles: u64,
    pub total_cycles: u64,
    pub hits: usize,
}

impl fmt::Display for CycleCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Hits: {}{}",
            self.hits,
            if self.hits > 0 {
                format!(
                    "\t\tAvg cycles: {}",
                    self.get_average_cycles().to_formatted_string(&Locale::en)
                )
            } else {
                String::default()
            }
        )
    }
}

impl CycleCounter {
    pub fn start(&mut self) {
        #[cfg(feature = "debug_cycle_counts")]
        {
            self.last_start_cycles = unsafe { _rdtsc() };
        }
    }

    pub fn end(&mut self) {
        #[cfg(feature = "debug_cycle_counts")]
        {
            unsafe {
                self.hits += 1;
                self.total_cycles += _rdtsc() - self.last_start_cycles;
            }
        }
    }

    fn get_average_cycles(&self) -> u64 {
        (self.total_cycles as f64 / self.hits as f64) as u64
    }
}
