pub mod part;
pub mod graph;

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
    fn default() -> Self {
        Signal::Error
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
}

impl Default for PinState {
    fn default() -> Self {
        PinState::HiZ
    }
}

impl Into<Signal> for PinState {
    fn into(self) -> Signal {
        match self {
            PinState::HiZ => Signal::Off,
            PinState::Input(signal) | PinState::Output(signal) => signal,
        }
    }
}
