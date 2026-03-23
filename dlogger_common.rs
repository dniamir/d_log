use core::sync::atomic::{AtomicU32, Ordering};

pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);

pub struct DLogger;

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

impl defmt::Format for DFmtI32 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

pub struct DFmtU32 { pub value: u32, pub precision: usize }

impl defmt::Format for DFmtU32 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

pub struct DFmtU16 { pub value: u16, pub precision: usize }

impl defmt::Format for DFmtU16 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{}", DFmt { value: self.value as i64, precision: self.precision });
    }
}

pub struct DFmtF32 { pub value: f32, pub precision: usize }

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
    #[inline] pub fn set_hold(count: usize) { DLOGGER_HOLD_COUNT.store(count as u32, Ordering::Relaxed); }
    #[inline] pub fn get_hold_count() -> usize { DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) as usize }
    #[inline] pub fn reset_hold() { DLOGGER_HOLD_COUNT.store(0u32, Ordering::Relaxed); }
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
