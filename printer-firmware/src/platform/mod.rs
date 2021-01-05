//! Platform should provide members for :
//! - rx : serial interface source of gcode
//! - tx : serial interface for debug messages

#[cfg(feature = "platform-nucleo-f401re")]
mod nucleo_f401re;

#[cfg(feature = "platform-disco-l475")]
mod disco_l475;

#[cfg(feature = "platform-nucleo-f401re")]
pub(crate) use nucleo_f401re::Platform;

#[cfg(feature = "platform-disco-l475")]
pub(crate) use disco_l475::Platform;
