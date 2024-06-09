use core::fmt::{self, Display};

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
            pad_right(self.hits.to_string(), 8),
            if self.hits > 0 {
                format!(
                    "Cycles/hit: {}",
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
        #[cfg(target_arch = "x86_64")]
        #[cfg(feature = "debug_cycle_counts")]
        {
            self.last_start_cycles = unsafe { _rdtsc() };
        }
    }

    pub fn end(&mut self) {
        #[cfg(target_arch = "x86_64")]
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

pub trait FromIndex {
    fn from_index(index: usize) -> Self;
}

#[derive(Default)]
pub struct CycleCounters([CycleCounter; 32]);

impl CycleCounters {
    pub fn get(&self, index: usize) -> &CycleCounter {
        // Panics on out-of-bounds.

        &self.0[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut CycleCounter {
        // Panics on out-of-bounds.

        &mut self.0[index]
    }

    pub fn reset(&mut self) {
        #[cfg(feature = "debug_cycle_counts")]
        {
            for counter in &mut self.0 {
                counter.hits = 0;
                counter.total_cycles = 0;
            }
        }
    }

    pub fn report<T: FromIndex + Display>(&self) {
        for (index, counter) in self.0.iter().enumerate() {
            if counter.last_start_cycles != 0 {
                let counter_label = format!("{}", T::from_index(index));
                let counter_label_padded = pad_right(counter_label, 20);

                println!("\t{}\t{}", counter_label_padded, counter);
            }
        }

        println!();
    }
}

fn pad_right(v: String, width: usize) -> String {
    let len = v.len();

    debug_assert!(width >= len);

    v + &" ".repeat(width - len)
}
