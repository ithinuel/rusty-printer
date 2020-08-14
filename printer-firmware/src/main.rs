#![no_std]
#![no_main]

extern crate gcode as egcode;
extern crate panic_halt;
//extern crate panic_semihosting;

mod executor;
mod gcode;
mod platform;

use arrayvec::ArrayVec;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use futures::{future, stream, StreamExt, TryStreamExt};

enum Positioning {
    Relative,
    Absolute,
}

enum MotionMode {
    RapidLinear,
    Linear,
    ClockwiseControledArc,
    CounterClockwiseControledArc,
    BezierCubicSpline,
    None,
}
/*,
    Homing,
    BedLeveling,
*/
enum Unit {
    Millimeter,
    Inch,
}

struct Workspace<T> {
    x: T,
    y: T,
    z: T,
}
enum Plane {
    XY,
    YZ,
    XZ,
}

//struct State {
//workspaces: [Workspace<f32>; 10],
//current_workspace: u8,
//place: Plane,
//unit: Unit,
//motion_mode: MotionMode,
//positioning: Positioning,
//axis_homed: Workspace<bool>,
//stepper_on: bool,
//hotend_temperature_target: Option<f32>,
//hotbed_temperature_target: Option<f32>,
//fan_speed: Option<f32>,
//}

#[derive(Debug)]
enum Error<IoError> {
    Io(IoError),
    Parsing(egcode::Error),
    InvalidLineNumber(u32),
    TooManyWords,
}
impl<IoError> From<egcode::Error> for Error<IoError> {
    fn from(e: egcode::Error) -> Self {
        Self::Parsing(e)
    }
}

struct SerialIterator<B> {
    reader: B,
    buffer: [u8; 128], // this would be nicer with a const generic parameter
    wr: usize,         // write ptr
    rd: usize,         // read ptr
}
impl<B: Read<u8>> SerialIterator<B> {
    fn new(reader: B) -> Self {
        Self {
            reader,
            buffer: [0; 128],
            wr: 0,
            rd: 127,
        }
    }
}
impl<B> core::fmt::Debug for SerialIterator<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        f.debug_struct("SerialIterator")
            .field("buffer", &self.buffer)
            .field("wr", &self.wr)
            .field("rd", &self.rd)
            .finish()
    }
}

impl<B: Read<u8>> Iterator for SerialIterator<B> {
    type Item = Result<u8, B::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.wr != self.rd {
            match self.reader.read() {
                Ok(byte) => {
                    self.buffer[self.wr] = byte;
                    self.wr += 1;
                    if self.wr == 128 {
                        self.wr = 0
                    }
                }
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(e)) => return Some(Err(e)),
            }
        }

        let mut next_rd = self.rd + 1;
        if next_rd == 128 {
            next_rd = 0;
        }
        if next_rd != self.wr {
            self.rd = next_rd;
            Some(Ok(self.buffer[next_rd]))
        } else {
            None
        }
    }
}

#[entry]
fn main() -> ! {
    use core::fmt::Write;

    let platform::Platform {
        sin: rx,
        sout: mut tx,
        name: platform_name,
    } = platform::Platform::take();

    executor::block_on(async move {
        let mut it = SerialIterator::new(rx);
        let strm = stream::poll_fn(|_| match it.next() {
            Some(b) => core::task::Poll::Ready(Some(b)),
            None => core::task::Poll::Pending,
        })
        .map(|res| res.map_err(Error::Io));
        let mut parser = egcode::Parser::new(strm);

        writeln!(
            tx,
            "Rusty ({}) VER:{} MODEL:{} HW:{}",
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_VERSION"),
            "from_config",
            platform_name
        )
        .unwrap_or(());
        // cap:<capability name in caps>:<0 or 1>
        //     AUTOREPORT_TEMP
        //     AUTOREPORT_SD_STATUS
        //     BUSY_PROTOCOL
        //     EMERGENCY_PARSER
        //     CHAMBER_TEMPERATURE
        //     Marlin/src/gcode/host/M115.cpp

        let mut next_line_number = 0;
        let mut error_recovery = false;
        loop {
            // take_until error or execute is reached or 10 elements have been picked
            let fut = stream::unfold(
                &mut parser,
                |p| async move { p.next().await.map(|w| (w, p)) },
            )
            .take_while(|res| {
                future::ready(match res {
                    Ok(egcode::GCode::Execute) => false,
                    _ => true,
                })
            })
            .take(10)
            .try_collect();

            //writeln!(tx, "wait: fut size: {}", core::mem::size_of_val(&fut)).unwrap_or(());
            let res_segments: Result<ArrayVec<[_; 10]>, Error<_>> = fut.await;

            //writeln!(tx, "wait: {} {:?}", had_error, res_segments).unwrap_or(());
            match res_segments {
                Ok(segments) if !error_recovery => {
                    if segments.len() == segments.capacity() {
                        // too many words
                        // discard the line
                        writeln!(
                            tx,
                            "error: Too many segment on the line (limit: {})",
                            segments.capacity()
                        )
                        .unwrap_or(());
                        error_recovery = true;
                    } else if !segments.is_empty() {
                        writeln!(tx, "ok {:?}", segments).unwrap_or(());
                        next_line_number += 1;
                    }
                }
                Ok(segments) => {
                    if segments.len() < 10 {
                        error_recovery = false;
                    }
                }
                Err(e) => {
                    writeln!(tx, "wait: Err({:?})", e).unwrap_or(());
                    writeln!(tx, "rs: N:{}", next_line_number).unwrap_or(());
                    error_recovery = true;
                }
            }

            // if err.is_some() ignore the words
            // if first == LineNumber => check line number
            // process words
        }
    });
    unreachable!()
}
