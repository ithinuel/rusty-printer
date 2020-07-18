#![no_std]
#![no_main]

extern crate gcode as egcode;
extern crate panic_halt;

mod gcode;
mod platform;

use arrayvec::ArrayVec;
use cortex_m_rt::entry;
use embedded_hal::serial::Read;

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

fn read<T, U, V>(
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
    use core::fmt::Write;

    let platform = platform::Platform::new();
    let mut state = egcode::Parser::new();

    let mut from_serial = platform.sin;
    let mut tx = platform.sout;

    let mut buffer: ArrayVec<[_; 10]> = Default::default();
    let mut words: ArrayVec<[_; 10]> = Default::default();

    let mut next_line_number = 0;
    let mut error_mode = false;

    //writeln!(tx, "start").unwrap_or(());

    loop {
        // fetch input as fast as possible
        let _ = read(
            &mut from_serial,
            &mut buffer,
            &mut state,
            &mut words,
            &mut error_mode,
        )
        .and_then(|_| {
            struct Consumer<'a, T>(&'a mut ArrayVec<T>)
            where
                T: arrayvec::Array<Item = u8>;
            impl<T> Iterator for Consumer<'_, T>
            where
                T: arrayvec::Array<Item = u8>,
            {
                type Item = u8;
                fn next(&mut self) -> Option<u8> {
                    self.0.pop()
                }
            }
            parse(
                Consumer(&mut buffer),
                &mut state,
                &mut words,
                next_line_number,
            )
        })
        .and_then(|_| {
            //writeln!(tx, "{:?} {:?} {:?}", buffer, state, words).unwrap_or(());
            process(&mut words, &mut next_line_number)
        })
        .map_err(|err| match err {
            nb::Error::Other(err) => {
                //writeln!(tx, "!! {:?}", err).unwrap_or(());
            }
            _ => {}
        });
    }
}
