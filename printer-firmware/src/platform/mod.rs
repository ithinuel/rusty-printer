//! Platform should provide members for :
//! - rx : serial interface source of gcode
//! - tx : serial interface for debug messages

#[cfg(feature = "platform-nucleo-f401re")]
mod nucleo_f401re;

#[cfg(feature = "platform-nucleo-f401re")]
pub(crate) use nucleo_f401re::Platform;
