//! Celestia utility layer: bit-flags, number formatting, file-type detection,
//! logging, high-precision fixed-point (`R128`), and string helpers.
//!
//! Faithful Rust port of parts of Celestia's `celutil` library:
//! `flag.h`, `formatnum.h`/`.cpp`, `filetype.h`/`.cpp`, `logger.h`/`.cpp`,
//! `r128.h`/`r128util.h`, `stringutils.h`/`.cpp`. The CEL data layer
//! (`Tokenizer`, `Value`, `Color`, `AssociativeArray`, the `.ssc`/`.cel`
//! parser) already lives in [`crate::celestia::cel_parser`]; this module
//! re-exports it so callers can treat `celutil` as the single `celutil`
//! equivalent.
//!
//! This module is intentionally free of any Bevy dependency so it can be
//! unit-tested in isolation and reused by the engine and the render layer.
#![allow(missing_docs)]

// Re-export the CEL data layer as the `celutil` equivalent.
pub use crate::celestia::cel_parser::{
    AssociativeArray, Color, Token, Tokenizer, Value,
};

// --- Bit flags (celutil/flag.h) ---------------------------------------------

/// A type whose values can be combined as bit flags, mirroring Celestia's
/// `ENUM_CLASS_BITWISE_OPS`/`ENUM_CLASS_BITSHIFT_OPS` and the free helpers
/// `is_set`/`set_or_unset`.
///
/// Celestia implements this with a macro over its `enum class` flag types; in
/// Rust the equivalent is a trait that any bit-flag enum implements.
pub trait Flag: Copy + Eq + std::fmt::Debug {
    /// The underlying integer type (e.g. `u32`).
    type Bits: std::ops::BitOr<Output = Self::Bits>
        + std::ops::BitAnd<Output = Self::Bits>
        + std::ops::BitXor<Output = Self::Bits>
        + std::ops::Shl<u32, Output = Self::Bits>
        + std::ops::Shr<u32, Output = Self::Bits>
        + std::ops::Not<Output = Self::Bits>
        + Copy;

    /// Convert to the underlying bit representation.
    fn to_bits(self) -> Self::Bits;

    /// Construct from the underlying bit representation.
    fn from_bits(bits: Self::Bits) -> Self;

    /// The "empty" flag (all bits clear), mirroring `static_cast<E>(0)`.
    fn empty() -> Self;

    /// `f | t`, mirroring `operator|`.
    #[inline]
    fn or(self, other: Self) -> Self {
        Self::from_bits(self.to_bits() | other.to_bits())
    }

    /// `f & t`, mirroring `operator&`.
    #[inline]
    fn and(self, other: Self) -> Self {
        Self::from_bits(self.to_bits() & other.to_bits())
    }

    /// `f ^ t`, mirroring `operator^`.
    #[inline]
    fn xor(self, other: Self) -> Self {
        Self::from_bits(self.to_bits() ^ other.to_bits())
    }

    /// `~f`, mirroring `operator~`.
    #[inline]
    fn not(self) -> Self {
        Self::from_bits(!self.to_bits())
    }

    /// `f | t`, mutating. Mirrors `operator|=`.
    #[inline]
    fn or_assign(&mut self, other: Self) {
        *self = self.or(other);
    }

    /// `f & t`, mutating. Mirrors `operator&=`.
    #[inline]
    fn and_assign(&mut self, other: Self) {
        *self = self.and(other);
    }

    /// Whether all bits of `target` are set in `self`. Mirrors `is_set`.
    #[inline]
    fn is_set(self, target: Self) -> bool {
        self.and(target) != Self::empty()
    }

    /// Set or clear the bits of `target`. Mirrors `set_or_unset`.
    #[inline]
    fn set_or_unset(&mut self, target: Self, set: bool) {
        if set {
            self.or_assign(target);
        } else {
            self.and_assign(target.not());
        }
    }
}

/// A `u32`-backed bit-flag, mirroring the common `enum class ... : unsigned int`
/// flags in Celestia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct U32Flag(pub u32);

impl Flag for U32Flag {
    type Bits = u32;
    fn to_bits(self) -> u32 {
        self.0
    }
    fn from_bits(bits: u32) -> Self {
        U32Flag(bits)
    }
    fn empty() -> Self {
        U32Flag(0)
    }
}

// --- Number formatting (celutil/formatnum.h / .cpp) -------------------------

/// Number-format options, mirroring `celutil::NumberFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NumberFormat(pub u32);

impl Flag for NumberFormat {
    type Bits = u32;
    fn to_bits(self) -> u32 {
        self.0
    }
    fn from_bits(bits: u32) -> Self {
        NumberFormat(bits)
    }
    fn empty() -> Self {
        NumberFormat(0)
    }
}

impl NumberFormat {
    /// No special formatting.
    pub const NONE: NumberFormat = NumberFormat(0);
    /// Insert thousands separators.
    pub const GROUP_THOUSANDS: NumberFormat = NumberFormat(1);
    /// Print a fixed number of significant figures.
    pub const SIGNIFICANT_FIGURES: NumberFormat = NumberFormat(2);
}

/// A minimal `NumberFormatter` that formats floating-point numbers with optional
/// thousands grouping and significant-figure precision. Mirrors the public
/// surface of `celutil::NumberFormatter` (the heavy `fmt`-based formatting is
/// replaced with `std` formatting, which is sufficient for HUD/label output).
#[derive(Debug, Clone, Copy, Default)]
pub struct NumberFormatter {
    precision: u32,
    format: NumberFormat,
}

impl NumberFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Format `value` with the default (thousands-grouped) style.
    pub fn format_value(&self, value: f64) -> String {
        let sig_figs = self.format.is_set(NumberFormat::SIGNIFICANT_FIGURES);
        let group = self.format.is_set(NumberFormat::GROUP_THOUSANDS);
        let precision = self.precision;
        format_number(value, precision, sig_figs, group)
    }
}

/// Format a floating-point number.
///
/// When `significant_figures` is set and `precision > 0`, the value is rounded to
/// `precision` significant figures; otherwise it is formatted with the given
/// decimal precision (0 => no decimal part). When `group_thousands` is set, the
/// integer part is separated by commas.
fn format_number(value: f64, precision: u32, significant_figures: bool, group_thousands: bool) -> String {
    let (whole, frac) = split_integer_decimal(value, group_thousands);
    if significant_figures && precision > 0 {
        // Round to `precision` significant figures and re-join.
        let sig = significant_figures_string(value, precision);
        return sig;
    }
    let decimals = format_decimal(value, precision);
    if group_thousands {
        format!("{whole}{decimals}")
    } else {
        format!("{value:.prec$}", prec = precision)
    }
}

/// Split a (possibly negative) value into its grouped integer string and a
/// decimal-fraction string (including the sign of the fractional part). Used by
/// the thousands-grouping path of [`format_number`].
fn split_integer_decimal(value: f64, group_thousands: bool) -> (String, String) {
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let int_part = abs.floor() as u64;
    let frac_part = abs - abs.floor();
    let int_str = if group_thousands {
        group_thousands(int_part)
    } else {
        int_part.to_string()
    };
    let frac_str = format_decimal_from_frac(frac_part, 6);
    if frac_str.is_empty() {
        (format!("{sign}{int_str}"), String::new())
    } else {
        (format!("{sign}{int_str}."), frac_str)
    }
}

fn format_decimal(value: f64, precision: u32) -> String {
    format!(".{:0>prec$}.0", "0", prec = precision as usize).trim_end_matches('.0').to_string()
}

fn format_decimal_from_frac(frac: f64, max_digits: u32) -> String {
    let digits: String = {
        let mut s = format!("{frac:.prec$}", prec = max_digits);
        s.trim_start_matches('0').trim_start_matches('.').to_string()
    };
    if digits.len() > max_digits as usize {
        digits.chars().take(max_digits as usize).collect()
    } else {
        digits
    }
}

/// Format `value` with `precision` significant figures.
fn significant_figures_string(value: f64, precision: u32) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let mut s = format!("{value:.prec$}", prec = (precision + 2));
    // Trim trailing zeros.
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

/// Insert comma thousands separators into a non-negative integer.
fn group_thousands(mut n: u64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::new();
    let remainder = bytes.len() % 3;
    for (i, b) in bytes.iter().enumerate() {
        out.push(*b as char);
        if i + 1 < bytes.len() {
            let pos = if remainder != 0 {
                i - remainder + 1
            } else {
                i + 1
            };
            if pos % 3 == 0 {
                out.push(',');
            }
        }
        n /= 10;
        if n == 0 {
            break;
        }
    }
    out
}

// --- File type detection (celutil/filetype.h / .cpp) -----------------------

/// A recognized Celestia content type, mirroring `celutil::ContentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ContentType {
    Jpeg = 1,
    Bmp = 2,
    Gif = 3,
    Png = 4,
    Targa = 5,
    CelestiaTexture = 6,
    _3DStudio = 7,
    CelestiaMesh = 8,
    Mkv = 9,
    CelestiaCatalog = 10,
    Dds = 11,
    CelestiaStarCatalog = 12,
    CelestiaDeepSkyCatalog = 13,
    CelestiaScript = 14,
    CelestiaLegacyScript = 15,
    CelestiaModel = 16,
    Dxt5NormalMap = 17,
    CelestiaXyzTrajectory = 18,
    CelestiaXyzvTrajectory = 19,
    WarpMesh = 21,
    CelestiaXyzvBinary = 22,
    Unknown = -1,
}

/// Determine the content type of a file from its name (or, when
/// `is_extension` is false, from its full path's extension). Mirrors
/// `celutil::DetermineFileType`.
///
/// This inspects the lowercase file extension and matches it against Celestia's
/// known extensions.
pub fn determine_file_type(filename: &str, is_extension: bool) -> ContentType {
    let ext = extension_of(filename, is_extension);
    match ext.as_str() {
        "jpg" | "jpeg" => ContentType::Jpeg,
        "bmp" => ContentType::Bmp,
        "gif" => ContentType::Gif,
        "png" => ContentType::Png,
        "tga" | "tpic" => ContentType::Targa,
        "tex" => ContentType::CelestiaTexture,
        "3ds" => ContentType::_3DStudio,
        "obj" => ContentType::CelestiaMesh,
        "mkv" => ContentType::Mkv,
        "ssc" | "cel" => ContentType::CelestiaCatalog,
        "dds" => ContentType::Dds,
        "stc" => ContentType::CelestiaStarCatalog,
        "dsc" => ContentType::CelestiaDeepSkyCatalog,
        "celx" => ContentType::CelestiaScript,
        "cel" => ContentType::CelestiaScript,
        "cmod" => ContentType::CelestiaModel,
        "nm16" => ContentType::Dxt5NormalMap,
        "xyz" => ContentType::CelestiaXyzTrajectory,
        "xyzv" => ContentType::CelestiaXyzvTrajectory,
        "bin" => ContentType::CelestiaXyzvBinary,
        _ => ContentType::Unknown,
    }
}

/// Extract the (lowercased) extension from a filename. When `is_extension` is
/// true, `filename` is already just the extension.
fn extension_of(filename: &str, is_extension: bool) -> String {
    if is_extension {
        return filename.to_lowercase();
    }
    match filename.rsplit('.').next() {
        Some(ext) => ext.to_lowercase(),
        None => String::new(),
    }
}

// --- Logging (celutil/logger.h / .cpp) --------------------------------------

/// A log level, mirroring `celutil::Level`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Info,
    Verbose,
    Debug,
}

/// A trivial file-or-`eprintln` logger. Mirrors the surface of
/// `celutil::Logger`; Celestia's version writes to a configurable stream with
/// `fmt` formatting, here replaced with the `log` crate / `eprintln`.
#[derive(Debug, Clone)]
pub struct Logger {
    prefix: String,
}

impl Logger {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    pub fn error(&self, msg: &str) {
        log::error!("[{}] {msg}", self.prefix);
    }

    pub fn warning(&self, msg: &str) {
        log::warn!("[{}] {msg}", self.prefix);
    }

    pub fn info(&self, msg: &str) {
        log::info!("[{}] {msg}", self.prefix);
    }

    pub fn verbose(&self, msg: &str) {
        log::debug!("[{}] {msg}", self.prefix);
    }

    pub fn debug(&self, msg: &str) {
        log::debug!("[{}] {msg}", self.prefix);
    }
}

// --- High-precision fixed point (celutil/r128.h / r128util.h) ---------------

/// A 128-bit signed fixed-point number with a 64.64 format (32 bits of fraction),
/// mirroring Celestia's `r128::R128`.
///
/// Celestia uses this type for the observer's high-precision interstellar
/// position, where the loss of precision from a plain `f64` is unacceptable.
/// The representation here is a `{lo, hi}` pair of `u64` forming a 128-bit
/// two's-complement value, scaled by `2^32` (matching Celestia's 64.64 with
/// `R128_SCALE = 32`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct R128 {
    lo: u64,
    hi: u64,
}

impl R128 {
    /// The number of fractional bits in the fixed-point representation.
    pub const SCALE: u32 = 32;

    /// The value `1` (i.e. `2^SCALE`).
    pub const ONE: R128 = R128 {
        lo: 1u64 << 32,
        hi: 0,
    };

    /// The value `0`.
    pub const ZERO: R128 = R128 { lo: 0, hi: 0 };

    /// Create from a raw `{lo, hi}` pair. Mirrors `R128(lo, hi)`.
    pub fn from_parts(lo: u64, hi: u64) -> Self {
        R128 { lo, hi }
    }

    /// Create from an `i64` integer. Mirrors `R128(R128_S64)`.
    pub fn from_i64(v: i64) -> Self {
        // Two's complement 128-bit representation, scaled by 2^32.
        let scaled = (v as u128) << 32;
        R128 {
            lo: scaled as u64,
            hi: (scaled >> 64) as u64,
        }
    }

    /// Create from an `f64`. Mirrors `R128(double)`.
    pub fn from_f64(v: f64) -> Self {
        // Round to the nearest representable value.
        let scaled = (v * (1u64 << 32) as f64).round() as i128;
        if scaled >= 0 {
            let u = scaled as u128;
            R128 {
                lo: u as u64,
                hi: (u >> 64) as u64,
            }
        } else {
            // Two's complement negation of 128-bit magnitude.
            let mag = (-scaled) as u128;
            let neg = (!mag).wrapping_add(1);
            R128 {
                lo: neg as u64,
                hi: (neg >> 64) as u64,
            }
        }
    }

    /// Convert to an `f64`, mirroring `operator double()`.
    pub fn to_f64(&self) -> f64 {
        // Reconstruct the 128-bit two's-complement integer and divide by 2^32.
        let lo = self.lo as i128;
        let hi = self.hi as i128;
        let full = (hi << 64) | (lo & (u64::MAX as i128));
        (full as f64) / (1u64 << 32) as f64
    }

    /// Add two `R128` values. Mirrors `operator+`.
    pub fn add(self, other: R128) -> R128 {
        let lo = self.lo as u128 + other.lo as u128;
        let carry = if lo >> 64 != 0 { 1 } else { 0 };
        R128 {
            lo: lo as u64,
            hi: (self.hi as u128 + other.hi as u128 + carry as u128) as u64,
        }
    }

    /// Subtract two `R128` values. Mirrors `operator-`.
    pub fn sub(self, other: R128) -> R128 {
        let lo = self.lo as u128 - other.lo as u128;
        let borrow = if self.lo as u128 < other.lo as u128 { 1 } else { 0 };
        R128 {
            lo: lo as u64,
            hi: (self.hi as u128 - other.hi as u128 - borrow as u128) as u64,
        }
    }
}

impl std::fmt::Display for R128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

// --- String utilities (celutil/stringutils.h / .cpp) -----------------------

/// Trim leading and trailing whitespace. Mirrors a common `celutil` string
/// helper.
pub fn trim(s: &str) -> &str {
    s.trim()
}

/// Return the substring after the first occurrence of `sep`, or `None`. Mirrors
/// the `celutil` "split after" helper used by the config file parser.
pub fn after_first(s: &str, sep: char) -> Option<&str> {
    s.find(sep).map(|i| &s[i + 1..])
}

/// Return the substring before the first occurrence of `sep`, or `None`.
pub fn before_first(s: &str, sep: char) -> Option<&str> {
    s.find(sep).map(|i| &s[..i])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_ops() {
        let mut f = U32Flag(0b0000);
        f.or_assign(U32Flag(0b0001));
        f.or_assign(U32Flag(0b0100));
        assert!(f.is_set(U32Flag(0b0001)));
        assert!(!f.is_set(U32Flag(0b0010)));
        f.set_or_unset(U32Flag(0b0001), false);
        assert!(!f.is_set(U32Flag(0b0001)));
        assert!(f.is_set(U32Flag(0b0100)));
    }

    #[test]
    fn number_format_flags() {
        let fmt = NumberFormat::GROUP_THOUSANDS.or(NumberFormat::SIGNIFICANT_FIGURES);
        assert!(fmt.is_set(NumberFormat::GROUP_THOUSANDS));
        assert!(fmt.is_set(NumberFormat::SIGNIFICANT_FIGURES));
    }

    #[test]
    fn file_type_detection() {
        assert_eq!(determine_file_type("solarsys.ssc", false), ContentType::CelestiaCatalog);
        assert_eq!(determine_file_type("stars.stc", false), ContentType::CelestiaStarCatalog);
        assert_eq!(determine_file_type("galaxies.dsc", false), ContentType::CelestiaDeepSkyCatalog);
        assert_eq!(determine_file_type("planet.3ds", false), ContentType::_3DStudio);
        assert_eq!(determine_file_type("model.cmod", false), ContentType::CelestiaModel);
        assert_eq!(determine_file_type("noext", false), ContentType::Unknown);
    }

    #[test]
    fn r128_round_trip() {
        let a = R128::from_f64(3.141592653589793);
        assert!((a.to_f64() - 3.141592653589793).abs() < 1e-12);
        let b = R128::from_i64(42);
        assert_eq!(b.to_f64(), 42.0);
        let sum = R128::from_f64(1.0).add(R128::from_f64(2.0));
        assert_eq!(sum.to_f64(), 3.0);
        let diff = R128::from_f64(5.0).sub(R128::from_f64(2.0));
        assert_eq!(diff.to_f64(), 3.0);
    }

    #[test]
    fn string_utils() {
        assert_eq!(trim("  hi  "), "hi");
        assert_eq!(after_first("key=val", '='), Some("val"));
        assert_eq!(before_first("key=val", '='), Some("key"));
        assert_eq!(after_first("noeq", '='), None);
    }
}
