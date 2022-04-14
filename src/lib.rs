pub mod gate;
pub mod graph;
pub mod ic;

use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub use gate::*;
pub use graph::*;
pub use ic::*;

/// The logical value for a given node, pin, etc.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum Signal {
    /// No signal present, high impedance, etc.
    Off,

    /// Logical Low (0, False, etc.)
    Low,

    /// Logical High (1, True, etc.)
    High,

    /// Uninitialized, indeterminate, or other problematic states
    ///
    /// Kind of like NaN in logical operations If any input is Error, then Error.
    Error,
}

impl Signal {
    /// Helper to treat Off and Low as the same value
    fn is_lowish(&self) -> bool {
        match self {
            Signal::Low | Signal::Off => true,
            _ => false,
        }
    }

    fn is_high(&self) -> bool {
        *self == Signal::High
    }
}

pub trait ToSignal {
    fn sig(&self) -> Signal;
}

impl ToSignal for Signal {
    fn sig(&self) -> Signal {
        *self
    }
}

impl ToSignal for &Signal {
    fn sig(&self) -> Signal {
        **self
    }
}

/// A collection of signals on a bus
///
/// BusValues convert to/from big-endian sequences of `Signals`
#[derive(Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct BusValue {
    /// Numerical representation the bus signals
    ///
    /// Bits will be 1 iff the corresponding `Signal` is `High`
    pub val: usize,

    /// Bit mask for signals in an `Error` state
    pub error: usize,
}

impl BusValue {
    /// Create a `BusValue` for the given number
    pub fn new_val(val: usize) -> Self {
        Self { val, error: 0 }
    }

    /// Create a `BusValue` with the given error mask
    pub fn new_error(error: usize) -> Self {
        Self { error, val: 0 }
    }

    /// If error mask is empty, returns the value; otherwise panics
    pub fn unwrap(&self) -> usize {
        assert_eq!(
            self.error, 0,
            "Attempting to unwrap BusValue with error_mask: {:#x}",
            self.error
        );

        self.val
    }

    /// Gets the `Signal` for the given bit position
    pub fn sig(&self, i: usize) -> Signal {
        if (self.error >> i) & 1 == 1 {
            Signal::Error
        } else if (self.val >> i) & 1 == 1 {
            Signal::High
        } else {
            Signal::Low
        }
    }
}

impl Debug for BusValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BusValue")
            .field("val", &format_args!("{:#x}", self.val))
            .field("error", &format_args!("{:#x}", self.error))
            .finish()
    }
}

/// Converts an object to a `BusValue`
///
/// Note: I originally tried implementing this as `impl Into<BusValue> for Foo`, but ran into
/// errors. Might revisit...
pub trait ToValue {
    fn val(self) -> BusValue;
}

impl ToValue for Signal {
    /// Creates a single-bit `BusValue` for a `Signal`
    fn val(self) -> BusValue {
        match self {
            Signal::Error => BusValue::new_error(1),
            Signal::High => BusValue::new_val(1),
            _ => BusValue::new_val(0),
        }
    }
}

impl<T> ToValue for T
where
    T: Iterator,
    T::Item: ToSignal,
{
    /// Creates a `BusValue` from a `ToSignal` iterator
    ///
    /// The iterator should be in big-endian order (least-significant first)
    fn val(self) -> BusValue {
        let mut bus_val = BusValue::default();
        for (i, sig) in self.map(|x| x.sig()).enumerate() {
            assert!(
                (i as u32) < usize::BITS,
                "Bus has more than usize::BITS ({}) bits",
                usize::BITS
            );
            let sig_val = sig.val();
            bus_val.val += sig_val.val << i;
            bus_val.error += sig_val.error << i;
        }

        bus_val
    }
}

impl Default for Signal {
    /// The default is Error to make it more obvious when things have not been connected correctly
    fn default() -> Signal {
        Signal::Error
    }
}

impl Not for Signal {
    type Output = Signal;

    /// Logical Not
    ///
    /// The behavior for Off is somewhat arbitrary. Logisim, e.g., returns Error.
    fn not(self) -> Signal {
        match self {
            Signal::High => Signal::Low,
            Signal::Low => Signal::High,
            Signal::Off => Signal::Off,
            _ => Signal::Error,
        }
    }
}

/// Pattern for matching a value with either element of a pair
macro_rules! either_are {
    ($val:path) => {
        ($val, _) | (_, $val)
    };
}

/// Pattern for matching a value with both elements of a pair
macro_rules! both_are {
    ($val:path) => {
        ($val, $val)
    };
}

/// Pattern for binding the unmatched element of a pair
macro_rules! other_is {
    ($val:path, $binding:ident) => {
        ($val, $binding) | ($binding, $val)
    };
}

impl BitAnd for Signal {
    type Output = Signal;

    /// Logical And
    ///
    /// Single Off treated as Low (pending implementation of Pull)
    fn bitand(self, rhs: Self) -> Signal {
        match (self, rhs) {
            either_are!(Signal::Error) => Signal::Error,
            both_are!(Signal::Off) => Signal::Off,
            both_are!(Signal::High) => Signal::High,
            _ => Signal::Low,
        }
    }
}

impl BitAndAssign for Signal {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl BitOr for Signal {
    type Output = Signal;

    /// Logical Or
    ///
    /// Single Off treated as Low (pending implementation of Pull)
    fn bitor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            either_are!(Signal::Error) => Signal::Error,
            other_is!(Signal::Off, a) => a,
            both_are!(Signal::Low) => Signal::Low,
            _ => Signal::High,
        }
    }
}
impl BitOrAssign for Signal {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitXor for Signal {
    type Output = Signal;

    /// Logical Or
    ///
    /// Single Off treated as Low (pending implementation of Pull)
    fn bitxor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            either_are!(Signal::Error) => Signal::Error,
            both_are!(Signal::Off) => Signal::Off,
            other_is!(Signal::High, a) if a != Signal::High => Signal::High,
            _ => Signal::Low,
        }
    }
}
impl BitXorAssign for Signal {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

/// The connection state and signal for a pin
///
/// Parts update their PinStates each tick. This allows connections to change between input, output,
/// and high impedance states (e.g. for chip enable, bidirectional ports, etc.)
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum PinState {
    /// High impedance, acting as neither an input nor an output. Logically Off
    HiZ,

    /// An input receives its signal from the connected Node
    Input(Signal),

    /// An output places its signal onto the connected Node
    ///
    /// If there is more than one non-Off output in a node, the node's signal will be `Error`
    Output(Signal),
}

impl PinState {
    /// Shorthand for a default input. Off until it gets a value from the Node.
    pub const INPUT: PinState = PinState::Input(Signal::Off);

    /// Shorthand for a default output. Error until the Part's updater runs.
    pub const OUTPUT: PinState = PinState::Output(Signal::Error);

    /// Helper for treating Off the same as Low
    pub fn is_lowish(&self) -> bool {
        self.sig().is_lowish()
    }

    pub fn is_high(&self) -> bool {
        self.sig().is_high()
    }
}

impl Default for PinState {
    fn default() -> Self {
        PinState::HiZ
    }
}

impl ToSignal for PinState {
    /// Get the logical signal for the PinState
    fn sig(&self) -> Signal {
        match self {
            PinState::HiZ => Signal::Off,
            PinState::Input(signal) | PinState::Output(signal) => *signal,
        }
    }
}

impl ToSignal for &PinState {
    fn sig(&self) -> Signal {
        (*self).sig()
    }
}

impl ToValue for &PinState {
    fn val(self) -> BusValue {
        self.sig().val()
    }
}

impl Into<Signal> for PinState {
    fn into(self) -> Signal {
        self.sig()
    }
}

impl Not for PinState {
    type Output = Signal;

    fn not(self) -> Signal {
        !self.sig()
    }
}

impl BitAnd for PinState {
    type Output = Signal;

    fn bitand(self, rhs: Self) -> Signal {
        self.sig() & rhs.sig()
    }
}

impl BitOr for PinState {
    type Output = Signal;

    fn bitor(self, rhs: Self) -> Signal {
        self.sig() | rhs.sig()
    }
}

impl BitXor for PinState {
    type Output = Signal;

    fn bitxor(self, rhs: Self) -> Signal {
        self.sig() ^ rhs.sig()
    }
}

#[cfg(test)]
#[test]
pub fn test_bus_val() {
    let mut sig_bus = [Signal::High; 5];

    assert_eq!(sig_bus.iter().val(), BusValue::new_val(31));

    sig_bus[2] = Signal::Error;

    assert_eq!(sig_bus.iter().val(), BusValue { val: 27, error: 4 });

    let mut state_bus = [
        PinState::HiZ,
        PinState::Output(Signal::High),
        PinState::Output(Signal::Off),
    ];

    assert_eq!(state_bus.iter().val(), BusValue::new_val(2));

    state_bus[0] = PinState::Output(Signal::Error);

    assert_eq!(state_bus.iter().val(), BusValue { val: 2, error: 1 });
}

#[cfg(test)]
mod test_signal {
    use crate::Signal::*;

    #[test]
    fn test_not() {
        assert_eq!(!Off, Off);
        assert_eq!(!Low, High);
        assert_eq!(!High, Low);
        assert_eq!(!Error, Error);
    }

    macro_rules! assert_error {
        ($op:tt) => {
            assert_eq!(Off $op Error, Error);
            assert_eq!(Low $op Error, Error);
            assert_eq!(High $op Error, Error);
            assert_eq!(Error $op Error, Error);
            assert_eq!(Error $op Off, Error);
            assert_eq!(Error $op Low, Error);
            assert_eq!(Error $op High, Error);
        }
    }

    #[test]
    fn test_and() {
        assert_error!(&);

        assert_eq!(Off & Off, Off);
        assert_eq!(Off & Low, Low);
        assert_eq!(Off & High, Low);

        assert_eq!(Low & Off, Low);
        assert_eq!(Low & Low, Low);
        assert_eq!(Low & High, Low);

        assert_eq!(High & Off, Low);
        assert_eq!(High & Low, Low);
        assert_eq!(High & High, High);
    }

    #[test]
    fn test_or() {
        assert_error!(|);

        assert_eq!(Off | Off, Off);
        assert_eq!(Off | Low, Low);
        assert_eq!(Off | High, High);

        assert_eq!(Low | Off, Low);
        assert_eq!(Low | Low, Low);
        assert_eq!(Low | High, High);

        assert_eq!(High | Off, High);
        assert_eq!(High | Low, High);
        assert_eq!(High | High, High);
    }

    #[test]
    fn test_xor() {
        assert_error!(^);

        assert_eq!(Off ^ Off, Off);
        assert_eq!(Off ^ Low, Low);
        assert_eq!(Off ^ High, High);

        assert_eq!(Low ^ Off, Low);
        assert_eq!(Low ^ Low, Low);
        assert_eq!(Low ^ High, High);

        assert_eq!(High ^ Off, High);
        assert_eq!(High ^ Low, High);
        assert_eq!(High ^ High, Low);
    }
}
