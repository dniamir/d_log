use core::sync::atomic::{AtomicU32, Ordering};

pub struct DLogger;

pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);

// Core formatting struct — all type-specific wrappers delegate here
pub struct DFmt {
    pub value: i64,
    pub precision: usize,
}

impl defmt::Format for DFmt {
    fn format(&self, f: defmt::Formatter) {
        let mut div = 1i64;
        for _ in 0..self.precision { div *= 10; }
        let whole = self.value / div;
        let frac = (self.value % div).abs();
        match self.precision {
            1 => defmt::write!(f, "{}.{:01}", whole, frac),
            2 => defmt::write!(f, "{}.{:02}", whole, frac),
            3 => defmt::write!(f, "{}.{:03}", whole, frac),
            4 => defmt::write!(f, "{}.{:04}", whole, frac),
            _ => defmt::write!(f, "{}.{}", whole, frac),
        }
    }
}

// Type-specific wrappers — value is a scaled integer (e.g. 2543 → 25.43 at precision 2)
pub struct DFmtI32 { pub value: i32, pub precision: usize }
pub struct DFmtU32 { pub value: u32, pub precision: usize }
pub struct DFmtU16 { pub value: u16, pub precision: usize }

// f32 wrapper — raw float, scaled internally
pub struct DFmtF32 { pub value: f32, pub precision: usize }

impl defmt::Format for DFmtI32 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

impl defmt::Format for DFmtU32 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

impl defmt::Format for DFmtU16 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

impl defmt::Format for DFmtF32 {
    fn format(&self, f: defmt::Formatter) {
        let mut scale = 1.0f32;
        for _ in 0..self.precision { scale *= 10.0; }
        defmt::write!(f, "{}", DFmt { value: (self.value * scale) as i64, precision: self.precision });
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
    // f32 branch: d_info!(f: "Temp: {} C", val, 2)
    (f: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger::DFmtF32 {
            value: $val as f32,
            precision: $prec,
        });
    };

    // Signed integer branch (i32/i16): d_info!(i: "Temp: {} °C", val, 2)
    (i: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger::DFmtI32 {
            value: $val as i32,
            precision: $prec,
        });
    };

    // Unsigned integer branch (u32/u16): d_info!(u: "Humidity: {} %", val, 2)
    (u: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger::DFmtU32 {
            value: $val as u32,
            precision: $prec,
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
    // f32 branch: d_force!(f: "CRITICAL Temp: {} C", val, 2)
    (f: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger::DFmtF32 {
            value: $val as f32,
            precision: $prec,
        });
    };

    // Signed integer branch: d_force!(i: "msg: {}", val, 2)
    (i: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger::DFmtI32 {
            value: $val as i32,
            precision: $prec,
        });
    };

    // Unsigned integer branch: d_force!(u: "msg: {}", val, 2)
    (u: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger::DFmtU32 {
            value: $val as u32,
            precision: $prec,
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