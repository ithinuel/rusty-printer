#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate gcode as egcode;
extern crate panic_halt;

mod executor;
mod gcode;
mod platform;

use arrayvec::ArrayVec;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
use futures::{future, stream, Stream, StreamExt};

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
enum Error<IoError, ParsingError> {
    Io(IoError),
    Parsing(ParsingError),
    InvalidLineNumber(u32),
    TooManyWords,
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

impl<B: Read<u8>> Iterator for SerialIterator<B> {
    type Item = Result<u8, B::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.wr != self.rd {
            match self.reader.read() {
                Ok(byte) => self.buffer[self.wr] = byte,
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

/*fn read<T, U, V>(
    from_serial: &mut dyn Read<u8, Error = U>,
    buffer: &mut ArrayVec<T>,
    state: &mut egcode::Parser,
    words: &mut ArrayVec<V>,
    error_mode: &mut bool,
) -> nb::Result<(), Error<U, egcode::Error>>
where
    T: arrayvec::Array<Item = u8>,
    V: arrayvec::Array<Item = egcode::GCode>,
{
    match from_serial.read() {
        Ok(byte) => {
            let is_eol = byte == b'\n' || byte == b'\r';
            if *error_mode {
                if is_eol {
                    *error_mode = false;
                    buffer.clear();
                    state.reset();
                    words.clear();
                }
            } else {
                let _ = buffer.try_push(byte);
                if buffer.is_full() || is_eol {
                    return Ok(());
                }
            }
            Err(nb::Error::WouldBlock)
        }
        Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
        Err(nb::Error::Other(err)) => {
            *error_mode = true;
            Err(nb::Error::from(Error::Io(err)))
        }
    }
}*/

/*fn parse<T, U, V: Iterator<Item = u8>>(
    buffer: V,
    state: &mut egcode::Parser,
    words: &mut ArrayVec<T>,
    next_line_number: u32,
) -> nb::Result<(), Error<U, egcode::Error>>
where
    T: arrayvec::Array<Item = egcode::GCode>,
{
    use egcode::GCode;

    // parse & cache segments
    let mut had_execute = false;
    let mut parser = state.parse(buffer);
    for res in &mut parser {
        match res {
            Ok(word) => {
                had_execute |= word == GCode::Execute;
                let _ = words.try_push(word);
            }
            Err(err) => return Err(nb::Error::from(Error::Parsing(err))),
        }
    }

    if had_execute {
        if words.is_full() && words.last().map(|w| w != &GCode::Execute).unwrap_or(true) {
            words.clear();
            Err(nb::Error::from(Error::TooManyWords))
        } else if words
            .first()
            .map(|w| match w {
                GCode::LineNumber(number) => *number != next_line_number,
                _ => false,
            })
            .unwrap_or(false)
        {
            words.clear();
            Err(nb::Error::from(Error::InvalidLineNumber(next_line_number)))
        } else {
            Ok(())
        }
    } else {
        Err(nb::Error::WouldBlock)
    }
}*/

/*fn process<T, U>(
    words: &mut ArrayVec<T>,
    next_line_number: &mut u32,
) -> nb::Result<(), Error<U, egcode::Error>>
where
    T: arrayvec::Array<Item = egcode::GCode>,
{
    use egcode::GCode;

    // The line is known to be valid so increment the next line number.
    *next_line_number += 1;
    if *next_line_number == 100_000 {
        *next_line_number = 0;
    }
    // process words, enqueue commands or execute them is not queueable
    for word in &*words {
        match word {
            GCode::Word(letter, value) => {
                // State change => XYZ motionmode movetype unit workspace plane
                //
            }
            _ => {}
        }
    }

    // process parameters
    for word in words {
        match word {
            GCode::ParameterSet(_id, _value) => {}
            _ => {}
        }
    }

    // clear word buffer.
    words.clear();
    Ok(())
}*/

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    loop {}
}

extern crate alloc;

use alloc_cortex_m::CortexMHeap;
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[entry]
fn main() -> ! {
    use core::fmt::Write;

    // Initialize the allocator BEFORE you use it
    let start = cortex_m_rt::heap_start() as usize;
    let size = 2048; // in bytes
    unsafe { ALLOCATOR.init(start, size) }

    let platform = platform::Platform::new();

    let mut rx = platform.sin;
    let mut tx = platform.sout;

    let mut words: ArrayVec<[egcode::GCode; 10]> = Default::default();

    let mut next_line_number = 0;
    let mut error_mode = false;

    //writeln!(tx, "start").unwrap_or(());
    let mut it = SerialIterator::new(rx);
    executor::block_on(async {
        let mut strm = stream::poll_fn(|_| match it.next() {
            Some(b) => core::task::Poll::Ready(Some(b)),
            None => core::task::Poll::Pending,
        })
        .filter_map(|res| {
            future::ready(match res {
                Ok(byte) => Some(byte),
                Err(e) => {
                    //writeln!(tx, "{:?}", e).unwrap_or(());
                    None
                }
            })
        });
        let parser = egcode::Parser::new(strm);
        let mut strm = stream::unfold(parser, |p| {
            let a = alloc::boxed::Box::pin(p.next());

            writeln!(tx, "{:?}", core::mem::size_of_val(&*a)).unwrap_or(());
            a
        });

        loop {
            let mut err = None;

            // take_until error or execute is reached or 10 elements have been picked
            let words: ArrayVec<[_; 10]> = strm
                .by_ref()
                .filter_map(|res| match res {
                    Ok(word) => future::ready(Some(word)),
                    Err(e) => {
                        err = Some(e);
                        future::ready(None)
                    }
                })
                .take_while(|word| future::ready(word != &egcode::GCode::Execute))
                .take(10)
                .collect()
                .await;
            //writeln!(tx, "{:?} {:?}", err, words).unwrap_or(());
        }
    });
    unreachable!()
}
