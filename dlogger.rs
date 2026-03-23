#[macro_export]
macro_rules! d_info {
    // f32 branch: d_info!(f: "Temp: {} C", val, 2)
    (f: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger_common::DFmtF32 {
            value: $val as f32,
            precision: $prec,
        });
    };

    // Signed integer branch (i32/i16): d_info!(i: "Temp: {} °C", val, 2)
    (i: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger_common::DFmtI32 {
            value: $val as i32,
            precision: $prec,
        });
    };

    // Unsigned integer branch (u32/u16): d_info!(u: "Humidity: {} %", val, 2)
    (u: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger_common::DFmtU32 {
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
        if $crate::d_log::dlogger_common::DLogger::allowed() {
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

            let count = $crate::d_log::dlogger_common::DLogger::get_hold_count();
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
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger_common::DFmtF32 {
            value: $val as f32,
            precision: $prec,
        });
    };

    // Signed integer branch: d_force!(i: "msg: {}", val, 2)
    (i: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger_common::DFmtI32 {
            value: $val as i32,
            precision: $prec,
        });
    };

    // Unsigned integer branch: d_force!(u: "msg: {}", val, 2)
    (u: $fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_force_internal!($fmt, $crate::d_log::dlogger_common::DFmtU32 {
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
