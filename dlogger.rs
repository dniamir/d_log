// d_info! — gated by DLogger::allowed(), indented by hold depth
//
//   d_info!("msg")                 — plain message, no args (interned format string)
//   d_info!(val)                   — single non-literal value: prints "{}" val
//   d_info!("fmt {}", val)         — format string + int (any defmt format specifier)
//   d_info!("fmt {}", val, prec)   — format string + float, rounded to `prec` decimal places
#[macro_export]
macro_rules! d_info {
    // Plain literal with no args: d_info!("message")
    // Must come before ($val:expr) — string literals are expressions and would otherwise
    // be routed there, causing defmt to serialize them as runtime &str values (triggering
    // TryFromIntError in defmt's export encoding) instead of interning them as format strings.
    ($fmt:literal) => {
        $crate::d_info_internal!($fmt);
    };

    // Single non-literal value: d_info!(val)
    ($val:expr) => {
        $crate::d_info_internal!("{}", $val);
    };

    // Float with precision: d_info!("label: {}", float_val, decimal_places)
    // Must come before the general fallback to avoid ambiguity on 3-token calls.
    ($fmt:literal, $val:expr, $prec:expr) => {
        $crate::d_info_internal!($fmt, $crate::d_log::dlogger_common::DFmtF32 {
            value: $val as f32,
            precision: $prec as usize,
        });
    };

    // Format string + int(s), any defmt specifier: d_info!("label: {}", val)
    ($fmt:literal, $($arg:tt)*) => {
        $crate::d_info_internal!($fmt, $($arg)*)
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

// d_force! — bypasses DLogger::allowed(), always logs
//
//   d_force!("msg")                — plain message, no args
//   d_force!(val)                  — single non-literal value
//   d_force!("fmt {}", val)        — format string + int
//   d_force!("fmt {}", val, prec)  — format string + float with precision
#[macro_export]
macro_rules! d_force {
    // Plain literal with no args — same fix as d_info!
    ($fmt:literal) => {
        defmt::info!($fmt)
    };

    // Single non-literal value
    ($val:expr) => {
        defmt::info!("{}", $val);
    };

    // Float with precision: d_force!("label: {}", float_val, decimal_places)
    ($fmt:literal, $val:expr, $prec:expr) => {
        defmt::info!($fmt, $crate::d_log::dlogger_common::DFmtF32 {
            value: $val as f32,
            precision: $prec as usize,
        });
    };

    // Format string + int(s), any defmt specifier
    ($fmt:literal, $($arg:tt)*) => {
        defmt::info!($fmt, $($arg)*);
    };
}
