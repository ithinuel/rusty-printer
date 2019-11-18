#![no_std]
#![no_main]

extern crate gcode as egcode;
extern crate panic_halt;

mod gcode;
mod platform;

use arrayvec::ArrayVec;
use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;
//use gcode_queue::GCodeQueue;

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
    BufferOverrun,
    Parsing(ParsingError),
    InvalidLineNumber(u32),
    TooManyWords,
}

fn read<T, U>(
    from_serial: &mut dyn Read<u8, Error = U>,
    buffer: &mut ArrayVec<T>,
    ignore_line: &mut bool,
) -> nb::Result<(), Error<U, egcode::Error>>
where
    T: arrayvec::Array<Item = u8>,
{
    match from_serial.read() {
        Ok(byte) => {
            if !buffer.is_full() {
                buffer.push(byte);
            }
            let is_eol = byte == b'\n' || byte == b'\r';
            if *ignore_line && is_eol {
                *ignore_line = false;
                buffer.clear();
            } else if buffer.is_full() || is_eol {
                return Ok(());
            }
            Err(nb::Error::WouldBlock)
        }
        Err(nb::Error::WouldBlock) => Err(nb::Error::WouldBlock),
        Err(nb::Error::Other(err)) => Err(nb::Error::from(Error::Io(err))),
    }
}

fn parse<T, U, V: Iterator<Item = u8>>(
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
    let mut parser = state.parse(buffer);
    for res in &mut parser {
        match res {
            Ok(word) => {
                words.push(word);
                if words.len() == words.capacity() {
                    break;
                }
            }
            Err(err) => return Err(nb::Error::from(Error::Parsing(err))),
        }
    }

    if !words
        .last()
        .map(|word| word == &GCode::Execute)
        .unwrap_or(false)
    {
        if words.len() == words.capacity() {
            Err(nb::Error::from(Error::TooManyWords))
        } else {
            Err(nb::Error::WouldBlock)
        }
    } else {
        if words
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
    }
}

fn process<T, U>(
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
    /*
     *for word in words {
     *    match word {
     *        GCode::ParameterSet(_id, _value) => {}
     *        _ => {}
     *    }
     *}
     */

    // clear word buffer.
    words.clear();
    Ok(())
}

#[entry]
fn main() -> ! {
    let platform = platform::Platform::new();
    let mut state = egcode::Parser::new();

    let mut from_serial = platform.sin;
    let mut tx = platform.sout;

    let mut buffer: ArrayVec<[_; 10]> = Default::default();
    let mut words: ArrayVec<[_; 10]> = Default::default();

    let mut next_line_number = 0;
    let mut ignore_line = false;

    loop {
        // fetch input as fast as possible
        read(&mut from_serial, &mut buffer, &mut ignore_line)
            .and_then(|_| parse(buffer.drain(..), &mut state, &mut words, next_line_number))
            .and_then(|_| {
                writeln!(tx, "{:?} {:?} {:?}", buffer, state, words).unwrap();
                process(&mut words, &mut next_line_number)
            })
            .map_err(|err| match err {
                nb::Error::Other(err) => {
                    writeln!(tx, "{:?}", err).unwrap();
                    ignore_line = true;
                    words.clear();
                    state.reset();
                }
                _ => {}
            });
    }
}
