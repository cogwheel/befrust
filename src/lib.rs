pub mod gate;
pub mod graph;
pub mod ic;

use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

pub use gate::*;
pub use graph::*;
pub use ic::*;

/// The logical value for a given node, pin, etc.
///
/// TODO: supporting busses might mean changing this to some kind of bitset
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum Signal {
    /// No signal present, high impedance, etc.
    Off, // TODO: maybe rename something else less confusing with Low

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

// TODO: surely there's a magic way to do this
impl ToSignal for &Signal {
    fn sig(&self) -> Signal {
        **self
    }
}

#[derive(Debug)]
pub struct BusValue {
    val: usize,
    error: usize,
}

impl BusValue {
    pub fn new_val(val: usize) -> Self {
        Self { val, error: 0 }
    }

    pub fn new_error(error: usize) -> Self {
        Self { error, val: 0 }
    }

    pub fn unwrap(&self) -> usize {
        assert_eq!(
            self.error, 0,
            "Attempting to unwrap BusValue with error_mask: {:#x}",
            self.error
        );

        self.val
    }
}

pub trait ToValue {
    fn val(self) -> BusValue;
}

impl ToValue for Signal {
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
    fn val(self) -> BusValue {
        let mut bus_val = BusValue { val: 0, error: 0 };
        for (i, sig) in self.map(|x| x.sig()).enumerate() {
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
    ///
    /// TODO: consider removing the ops for signals if the behavior will depend on the pin's pull
    fn not(self) -> Signal {
        match self {
            Signal::High => Signal::Low,
            Signal::Low => Signal::High,
            Signal::Off => Signal::Off, // TODO: support pull
            _ => Signal::Error,
        }
    }
}

macro_rules! either_are {
    ($val:path) => {
        ($val, _) | (_, $val)
    };
}

macro_rules! both_are {
    ($val:path) => {
        ($val, $val)
    };
}

macro_rules! one_is {
    ($val:path, $binding:ident) => {
        ($val, $binding) | ($binding, $val)
    };
}

impl BitAnd for Signal {
    type Output = Signal;

    /// Logical And
    ///
    /// Single Off treated as Low
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
    /// Single Off treated as Low
    fn bitor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            either_are!(Signal::Error) => Signal::Error,
            one_is!(Signal::Off, a) => a,
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
    /// Single Off treated as Low
    fn bitxor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            either_are!(Signal::Error) => Signal::Error,
            both_are!(Signal::Off) => Signal::Off,
            one_is!(Signal::High, a) if a != Signal::High => Signal::High,
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
///
// TODO: rename to Port? might confuse with Part
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum PinState {
    /// High impedance, acting as neither an input nor an output. Logically Off
    HiZ,

    /// An input receives its signal from the connected Node
    Input(Signal), // TODO: PullUp/Down

    /// An output places its signal onto the connected Node
    Output(Signal),
}

impl PinState {
    /// Shorthand for a default input. Off until it gets a value from the Node.
    pub const INPUT: PinState = PinState::Input(Signal::Off);

    /// Shorthand for a default output. Error until the Part's updater runs.
    pub const OUTPUT: PinState = PinState::Output(Signal::Error);

    /// Helper for treating Off the same as Low
    pub fn is_lowish(&self) -> bool {
        [Signal::Low, Signal::Off].contains(&self.sig())
    }

    /// TODO: add others?
    pub fn is_high(&self) -> bool {
        self.sig() == Signal::High
    }

    pub fn val(&self) -> BusValue {
        self.sig().val()
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

impl Default for PinState {
    fn default() -> Self {
        PinState::HiZ
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

    assert_eq!(sig_bus.iter().val(), Ok(31));

    sig_bus[2] = Signal::Error;

    assert_eq!(sig_bus.iter().val(), Err(()));

    let mut state_bus = [
        PinState::HiZ,
        PinState::Output(Signal::High),
        PinState::Output(Signal::Off),
    ];

    assert_eq!(state_bus.iter().val(), Ok(2));

    state_bus[0] = PinState::Output(Signal::Error);

    assert_eq!(state_bus.iter().val(), Err(()));
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
