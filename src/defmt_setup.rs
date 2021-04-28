use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_rtt as _; // global logger
use panic_probe as _;

pub use defmt::*;

/* #[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
} */

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});
