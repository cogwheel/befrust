pub mod part;
pub mod graph;

use std::ops::{BitAnd, BitOr, BitXor, Not};

pub use part::*;
pub use graph::*;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum Signal {
    Off,
    Low,
    High,
    Error,
}

impl Signal {
    pub const fn is_on(&self) -> bool {
        match self {
            Signal::Low | Signal::High => true,
            _ => false,
        }
    }

    pub const fn not(&self) -> Signal {
        match *self {
            Signal::Low => Signal::High,
            Signal::High => Signal::Low,
            _ => Signal::Error,
        }
    }
}

impl Default for Signal {
    fn default() -> Signal {
        Signal::Error
    }
}

impl Not for Signal {
    type Output = Signal;

    fn not(self) -> Signal {
        match self {
            Signal::High => Signal::Low,
            Signal::Low => Signal::High,
            Signal::Off => Signal::Off, // TODO: support pull?
            _ => Signal::Error,
        }
    }
}

impl BitAnd for Signal {
    type Output = Signal;

    fn bitand(self, rhs: Self) -> Signal {
        match (self, rhs) {
            (Signal::Error, _) | (_, Signal::Error) => Signal::Error,
            (Signal::Off, Signal::Off) => Signal::Off,
            (Signal::High, Signal::High) => Signal::High,
            _ => Signal::Low,
        }
    }
}


impl BitOr for Signal {
    type Output = Signal;

    fn bitor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            (Signal::Error, _) | (_, Signal::Error) => Signal::Error,
            (Signal::Off, a) | (a, Signal::Off) => a,
            (Signal::Low, Signal::Low) => Signal::Low,
            _ => Signal::High,
        }
    }
}

impl BitXor for Signal {
    type Output = Signal;

    fn bitxor(self, rhs: Self) -> Signal {
        match (self, rhs) {
            (Signal::Error, _) | (_, Signal::Error) => Signal::Error,
            (Signal::Off, Signal::Off) => Signal::Off,
            (a, Signal::High) | (Signal::High, a) if a != Signal::High => Signal::High,
            _ => Signal::Low,
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Hash)]
pub enum PinState {
    HiZ,
    Input(Signal), // TODO: PullUp/Down
    Output(Signal),
}

impl PinState {
    pub const INPUT: PinState = PinState::Input(Signal::Off);
    pub const OUTPUT: PinState = PinState::Output(Signal::Error);

    pub fn signal(self) -> Signal {
        self.into()
    }
}

impl Default for PinState {
    fn default() -> Self {
        PinState::HiZ
    }
}

impl Into<Signal> for PinState {  // TODO: rename Port? might confuse with Part...
    fn into(self) -> Signal {
        match self {
            PinState::HiZ => Signal::Off,
            PinState::Input(signal) | PinState::Output(signal) => signal,
        }
    }
}

impl Not for PinState {
    type Output = Signal;

    fn not(self) -> Signal {
        !self.signal()
    }
}

impl BitAnd for PinState {
    type Output = Signal;

    fn bitand(self, rhs: Self) -> Signal {
        self.signal() & rhs.signal()
    }
}

impl BitOr for PinState {
    type Output = Signal;

    fn bitor(self, rhs: Self) -> Signal {
        self.signal() | rhs.signal()
    }
}

impl BitXor for PinState {
    type Output = Signal;

    fn bitxor(self, rhs: Self) -> Signal {
        self.signal() ^ rhs.signal()
    }
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