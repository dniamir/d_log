use core::sync::atomic::{AtomicU32, Ordering};

// Hold counts packed as 4-bit nibbles: bits [task_id*4 .. task_id*4+3].
// Supports up to 8 task IDs (0–7), each with a hold depth of 0–15.
pub static DLOGGER_HOLD_COUNT: AtomicU32 = AtomicU32::new(0);
pub static ACTIVE_TASK_ID: AtomicU32 = AtomicU32::new(0);

pub struct DLogger;

// Core fixed-point formatter — stores a scaled integer and a precision (decimal places)
pub struct DFmt {
    pub value: i32,
    pub precision: usize,
}

impl defmt::Format for DFmt {
    fn format(&self, f: defmt::Formatter) {
        let div = 10i32.pow(self.precision as u32);
        
        // Now both are i64, so the compiler is happy
        let whole = self.value / div;
        let remainder = self.value % div;
        
        let frac = if remainder < 0 { -remainder } else { remainder };

        match self.precision {
            0 => defmt::write!(f, "{}", whole),
            1 => defmt::write!(f, "{}.{:01}", whole, frac),
            2 => defmt::write!(f, "{}.{:02}", whole, frac),
            3 => defmt::write!(f, "{}.{:03}", whole, frac),
            4 => defmt::write!(f, "{}.{:04}", whole, frac),
            _ => defmt::write!(f, "{}.{}", whole, frac),
        }
    }
}

// Float formatter — scales an f32 into a DFmt for fixed-decimal output
pub struct DFmtF32 { pub value: f32, pub precision: usize }

impl defmt::Format for DFmtF32 {
    fn format(&self, f: defmt::Formatter) {
        let mut scale = 1.0f32;
        for _ in 0..self.precision { scale *= 10.0; }
        defmt::write!(f, "{}", DFmt { value: (self.value * scale) as i32, precision: self.precision });
    }
}

impl DLogger {
    // Returns the bit shift for the current task's 4-bit nibble.
    #[inline] fn shift() -> u32 { ACTIVE_TASK_ID.load(Ordering::Relaxed) * 4 }

    // Increment the current task's hold depth (saturates at 15).
    #[inline] pub fn hold() {
        let shift = Self::shift();
        let mask = 0xFu32 << shift;
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            let nibble = (v >> shift) & 0xF;
            Some(if nibble < 0xF { (v & !mask) | ((nibble + 1) << shift) } else { v })
        }).ok();
    }

    // Decrement the current task's hold depth (floors at 0).
    #[inline] pub fn release() {
        let shift = Self::shift();
        let mask = 0xFu32 << shift;
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            let nibble = (v >> shift) & 0xF;
            Some(if nibble > 0 { (v & !mask) | ((nibble - 1) << shift) } else { v })
        }).ok();
    }

    // Set the current task's hold depth to an explicit value (capped at 15).
    #[inline] pub fn set_hold(count: usize) {
        let shift = Self::shift();
        let mask = 0xFu32 << shift;
        let nibble = (count as u32).min(0xF);
        DLOGGER_HOLD_COUNT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            Some((v & !mask) | (nibble << shift))
        }).ok();
    }

    // Read the current task's hold depth (0–15).
    #[inline] pub fn get_hold_count() -> usize {
        let shift = Self::shift();
        ((DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) >> shift) & 0xF) as usize
    }

    // Clear the current task's hold depth to 0.
    #[inline] pub fn reset_hold() {
        let shift = Self::shift();
        DLOGGER_HOLD_COUNT.fetch_and(!(0xFu32 << shift), Ordering::Relaxed);
    }

    // True when the current task's hold depth is 0.
    #[inline] pub fn allowed() -> bool {
        #[cfg(feature = "no_hold")] { true }
        #[cfg(not(feature = "no_hold"))] {
            let shift = Self::shift();
            (DLOGGER_HOLD_COUNT.load(Ordering::Relaxed) >> shift) & 0xF == 0
        }
    }

    #[inline] pub fn d_sep() { defmt::info!("======================="); }
    #[inline] pub fn d_restart() {
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
        defmt::info!("*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*=*");
    }
}
