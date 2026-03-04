use core::sync::atomic::{AtomicU32, Ordering};

pub struct DLogger;

pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);

// REMOVED: #[derive(defmt::Format)] - You cannot have both derive and manual impl
pub struct DFmt {
    pub value: f32,
    pub precision: usize,
}

impl defmt::Format for DFmt {
    fn format(&self, f: defmt::Formatter) {
        let integrity = self.value as i32;
        let mut power = 1.0;
        for _ in 0..self.precision {
            power *= 10.0;
        }
        let fractional = ((self.value - integrity as f32) * power).abs() as i32;
        
        match self.precision {
            1 => defmt::write!(f, "{}.{:01}", integrity, fractional),
            2 => defmt::write!(f, "{}.{:02}", integrity, fractional),
            3 => defmt::write!(f, "{}.{:03}", integrity, fractional),
            4 => defmt::write!(f, "{}.{:04}", integrity, fractional),
            _ => defmt::write!(f, "{}.{}", integrity, fractional),
        }
    }
}

impl DLogger {
    #[inline] pub fn hold() { DLOGGER_HOLD_COUNT.fetch_add(1, Ordering::Relaxed); }
    #[inline] pub fn release() {
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            if x > 0 { Some(x - 1) } else { Some(0) }
        }).ok();
    }
    #[inline] pub fn get_hold_count() -> usize { DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) as usize }
    #[inline] pub fn allowed() -> bool {
        #[cfg(feature = "no_hold")] { true }
        #[cfg(not(feature = "no_hold"))] { DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) == 0 }
    }
    #[inline] pub fn d_sep() { defmt::info!("======================="); }
    #[inline] pub fn d_restart() {
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
    }
}

#[macro_export]
macro_rules! d_info {
    // Specialized float branch: We use a specific prefix to avoid matching standard logs
    // Call this via: d_info!(f: "Temp: {} C", val, 2)
    (f: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger::DFmt { 
            value: $val, 
            precision: $prec 
        });
    };

    // Standard fallback: This now safely handles any number of arguments
    ($($arg:tt)*) => {
        $crate::d_info_internal!($($arg)*);
    };
}

#[macro_export]
macro_rules! d_info_internal {
    ($($arg:tt)*) => {
        if $crate::d_log::dlogger::DLogger::allowed() {
            struct LogProxy<'a>(usize, &'a dyn Fn(defmt::Formatter<'_>));

            impl<'a> defmt::Format for LogProxy<'a> {
                fn format(&self, fmt: defmt::Formatter<'_>) {
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
                    (self.1)(fmt);
                }
            }

            let count = $crate::d_log::dlogger::DLogger::get_hold_count();
            defmt::info!("{:?}", LogProxy(count, &|fmt| {
                defmt::write!(fmt, $($arg)*);
            }));
        }
    };
}

#[macro_export]
macro_rules! d_force {
    // 1. Specialized float branch for forced logs
    // Usage: d_force!(f: "CRITICAL Temp: {} C", val, 2)
    (f: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger::DFmt { 
            value: $val, 
            precision: $prec 
        });
    };

    // 2. Standard fallback for forced logs
    ($($arg:tt)*) => {
        $crate::d_force_internal!($($arg)*);
    };
}

#[macro_export]
macro_rules! d_force_internal {
    ($($arg:tt)*) => {
        // Skips the DLogger::allowed() check entirely
        defmt::info!($($arg)*);
    };
}