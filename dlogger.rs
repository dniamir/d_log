use core::sync::atomic::{AtomicU32, Ordering};

pub struct DLogger;

// Global flag lives OUTSIDE the impl
pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);

impl DLogger {
    #[inline]
    pub fn hold() {
        // Always track depth regardless of feature [cite: 1761, 1803]
        DLOGGER_HOLD_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn release() {
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            if x > 0 { Some(x - 1) } else { Some(0) }
        }).ok();
    }

    #[inline]
    pub fn get_hold_count() -> usize {
        DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) as usize
    }

    #[inline]
    pub fn allowed() -> bool {
        #[cfg(feature = "no_hold")]
        { true } // Always allow if feature is on [cite: 41, 1554]
        #[cfg(not(feature = "no_hold"))]
        { DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) == 0 }
    }

    #[inline]
    pub fn d_sep() {
        defmt::info!("=======================");
    }
    #[inline]
    pub fn d_restart() {
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
    }
}

#[macro_export]
macro_rules! d_info {
    ($($arg:tt)*) => {
        if $crate::d_log::dlogger::DLogger::allowed() {
            #[cfg(feature = "no_hold")]
            {
                // 1. Change signature to take `defmt::Formatter` by value (it's Copy)
                struct LogProxy<'a>(usize, &'a dyn Fn(defmt::Formatter<'_>));

                impl<'a> defmt::Format for LogProxy<'a> {
                    fn format(&self, fmt: defmt::Formatter<'_>) {
                        // 2. Write the prefix to the formatter handle
                        match self.0 {
                            0 => {},
                            1 => defmt::write!(fmt, "-"),
                            2 => defmt::write!(fmt, "--"),
                            3 => defmt::write!(fmt, "---"),
                            4 => defmt::write!(fmt, "----"),
                            5 => defmt::write!(fmt, "-------"),
                            6 => defmt::write!(fmt, "---------"),
                            7 => defmt::write!(fmt, "-----------"),
                            8 => defmt::write!(fmt, "-------------"),
                            9 => defmt::write!(fmt, "---------------"),
                            _ => defmt::write!(fmt, "-----------------+"),
                        };
                        // 3. Call closure with the formatter handle directly
                        (self.1)(fmt);
                    }
                }

                let count = $crate::d_log::dlogger::DLogger::get_hold_count();
                // 4. Pass `fmt` by value in the closure args
                defmt::info!("{:?}", LogProxy(count, &|fmt| {
                    defmt::write!(fmt, $($arg)*);
                }));
            }
            #[cfg(not(feature = "no_hold"))]
            {
                defmt::info!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! d_force {
    ($($arg:tt)*) => {
        defmt::info!($($arg)*);
    };
}
