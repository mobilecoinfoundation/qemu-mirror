#![deny(missing_docs)]

//! A binary that parses qemu -d output, and looks for different traces for the
//! same function.

// We expect to parse from STDIN a stream like the following, from qemu.
/*
----------------
IN: _ZN7rand_hc5hc1289Hc128Core4init2f217h0cae54e07949bfdfE
0x0000004000089d80:  
OBJD-T: 4883ec28897c2404897c240c897c241cc744242011000000c4e37bf0c7118944
OBJD-T: 24248b442424894424088b44240489442410c744241413000000c4e37bf0c013
OBJD-T: 894424188b4424188904248b4c24048b14248b44240831d0c1e90a31c84883c4
OBJD-T: 28c3

----------------
IN: _ZN68_$LT$rand_hc..hc128..Hc128Core$u20$as$u20$rand_core..SeedableRng$GT$9from_seed17he3db30f16bee3188E
0x000000400008a58e:  
OBJD-T: 488b4424388b4c242c488b5424308b949428010000898c24a0220000899424a4
OBJD-T: 22000001d1894c241c4883e80f4889442420483d000400000f92c0a801751e

----------------
IN: _ZN68_$LT$rand_hc..hc128..Hc128Core$u20$as$u20$rand_core..SeedableRng$GT$9from_seed17he3db30f16bee3188E
0x000000400008a5eb:  
OBJD-T: 488b4424208bbc8428010000e814f7ffff

----------------
IN: _ZN7rand_hc5hc1289Hc128Core4init2f117h9b8ae5045fdecaa8E
0x0000004000089d10:  
OBJD-T: 4883ec28897c2404897c240c897c241cc744242007000000c4e37bf0c7078944
OBJD-T: 24248b442424894424088b44240489442410c744241412000000c4e37bf0c012
OBJD-T: 894424188b4424188904248b4c24048b14248b44240831d0c1e90331c84883c4
OBJD-T: 28c3
*/
// Here OBJD-T: is hex-encoded object code for the target arch (here x86-64-linux-musl)
//
// Sometimes a function name will appear several times with different addresses.
// This happens if the function has multiple basic blocks inside it, for instance if it is a for loop.

use displaydoc::Display;
use env_logger::{fmt::Color, Builder, Env};
use log::Level;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env,
    io::{BufRead, Write},
};

/// Data that we parsed from a translation block record from qemu.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct TranslationBlock{
    /// The mangled name of a function associated to (containing) the translation block.
    pub mangled_name: String,
    /// The address (program counter) of the basic block.
    pub address: u64,
    /// Object dump. This is the concatenation of all the hex-encoded OBJD-T sections.
    pub objd_t_hex: String,
}

/// An error which occurs when trying to read a translation block
#[derive(Debug, Display)]
pub enum TBReadError {
    /// IO: {0}
    Io(std::io::Error),
    /// Parse int error: {0}
    ParseInt(std::num::ParseIntError),
    /// Expected Dashes to start a TB report, found {0}
    ExpectedDashes(String),
    /// Expected IN: segment, found {0}
    ExpectedInSegment(String),
    /// Expected Address, found {0}
    ExpectedAddress(String),
    /// Expected OBJD-T, found {0}
    ExpectedObjdt(String),
    /// Expected OBJD-T or skipped line, found {0}
    ExpectedObjdtOrSkip(String),
    /// Unexpected EOF
    UnexpectedEof,
}

impl From<std::io::Error> for TBReadError {
    fn from(src: std::io::Error) -> Self {
        Self::Io(src)
    }
}

impl From<std::num::ParseIntError> for TBReadError {
    fn from(src: std::num::ParseIntError) -> Self {
        Self::ParseInt(src)
    }
}

fn read_translation_block(reader: &mut impl BufRead) -> Result<Option<TranslationBlock>, TBReadError> {
    let mut tb = TranslationBlock::default();

    let mut buf = String::new();

    let result = reader.read_line(&mut buf)?;
    // If result is Ok(0) then we have reached EOF
    if result == 0 { return Ok(None); }

    if buf.trim_end() != "----------------" {
        return Err(TBReadError::ExpectedDashes(buf));
    }

    buf.clear();
    let result = reader.read_line(&mut buf)?;
    if result == 0 { return Err(TBReadError::UnexpectedEof); }

    if !buf.starts_with("IN:") {
        return Err(TBReadError::ExpectedInSegment(buf));
    }
    tb.mangled_name = buf.trim()[3..].to_string();

    buf.clear();
    let result = reader.read_line(&mut buf)?;
    if result == 0 { return Err(TBReadError::UnexpectedEof); }

    if !buf.starts_with("0x") {
        return Err(TBReadError::ExpectedAddress(buf));
    }
    tb.address = u64::from_str_radix(&buf.trim_end()[2..18], 16)?;

    buf.clear();
    let result = reader.read_line(&mut buf)?;
    if result == 0 { return Err(TBReadError::UnexpectedEof); }
    if !buf.starts_with("OBJD-T: ") {
        return Err(TBReadError::ExpectedObjdt(buf));
    }
    
    tb.objd_t_hex = buf.trim_end()[8..].to_string();

    loop {
        buf.clear();
        let result = reader.read_line(&mut buf)?;
        // This means the file has ended, we think that's okay at this point.
        if result == 0 { return Ok(Some(tb)); }
        // This is an empty new line after the objdt record, which signals the end of the objdt record.
        if buf.trim_end().is_empty() { return Ok(Some(tb)); }
        // Otherwise, we expect an additional objd-t record.
        if !buf.starts_with("OBJD-T: ") {
            return Err(TBReadError::ExpectedObjdtOrSkip(buf));
        }
        tb.objd_t_hex.push_str(&buf.trim_end()[8..]);
    }
}

fn record_objdt(known_objdts: &mut HashMap<String, HashSet<String>>, mangled_name: String, objdt: String) {
    known_objdts.entry(mangled_name).or_default().insert(objdt);
}


fn main() {
    make_basic_logger();

    let mut args = std::env::args();
    if args.len() > 2 || (args.len() == 2 && args.nth(1).unwrap() == "--help") {
        eprintln!("Usage:");
        eprintln!("Stream output of qemu -d in_asm into this program on stdin.");
        eprintln!("If desired, pass a filter string, such as a rust crate name, so that only symbols containing that string will be tracked.");
        eprintln!("");
        eprintln!("Example:");
        eprintln!("qemu-x86_64 -d in_asm,nochain rust_test_target --test-threads=1 2> >(mc-read-trace my-crate-name)");
    }

    let target_str = std::env::args().nth(1);

    if let Some(target_str) = target_str.as_ref() {
        eprintln!("Parsing in_asm blocks, with target_str = {}", target_str);
    } else {
        eprintln!("Parsing in_asm blocks, with no target_str");
    }

    let filter = |mangled_name: &str| -> bool {
        if mangled_name.is_empty() { return false; }
        if let Some(target_str) = target_str.clone() {
            if !mangled_name.contains(&target_str) { return false; }
        }
        for ignore in ["6access", "6create", "4main", "3new", "4test", "7testing"] {
            if mangled_name.contains(ignore) { return false; }
        }
        true
    };

    let mut prev_mangled_name: Option<String> = None;
    let mut running_objdt = String::new();
    
    let mut known_objdts = HashMap::<String, HashSet<String>>::default();

    let mut stdin = std::io::stdin().lock();
    loop {
        match read_translation_block(&mut stdin) {
            Ok(None) => break,
            Ok(Some(tb)) => {
                if prev_mangled_name == Some(tb.mangled_name.clone()) {
                    running_objdt.push_str(&tb.objd_t_hex);
                } else {
                    // At this point, we have to record prev_mangled_name and running_objdt in the known_objdts before doing anything else.
                    // Only track objdts for mangled names that pass the filter
                    if let Some(prev_mangled_name) = prev_mangled_name {
                        if filter(&prev_mangled_name) {
                            record_objdt(&mut known_objdts, prev_mangled_name, running_objdt);
                        } else {
                            log::debug!("Skipping symbol: {}", prev_mangled_name);
                        }
                    }

                    // We've now taken care of previous state.
                    prev_mangled_name = Some(tb.mangled_name);
                    running_objdt = tb.objd_t_hex;
                }
            }
            Err(err) => {
                log::error!("Parsing: {}", err);
                
            }
        }
    }

    log::info!("Finished parsing. Found {} symbols.", known_objdts.len());
    log::info!("The following symbols had multiple distinct traces:");
    let multiples: BTreeMap<String, BTreeSet<String>> = known_objdts.iter().filter(|(_mangled_name, traces)| traces.len() > 1).map(|(name, collection)| (name.clone(), collection.into_iter().cloned().collect())).collect();

    for (name, traces) in &multiples {
        log::info!("Symbol {} has {} distinct traces:", name, traces.len());
    }
    if multiples.is_empty() {
        log::info!("No symbols had more than one distinct trace.");
    }
}

fn make_basic_logger() {
    // Support LOG in addition to RUST_LOG. This allows us to not affect
    // cargo's logs when doing stuff like LOG=trace cargo test -p ...
    if env::var("RUST_LOG").is_err() && env::var("LOG").is_ok() {
        env::set_var("RUST_LOG", env::var("LOG").unwrap());
    }
    // Default to INFO log level for everything if we do not have an explicit
    // setting.
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let mut style = buf.style();

            let color = match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Cyan,
                Level::Trace => Color::Magenta,
            };
            style.set_color(color).set_bold(true);

            // Drop cargo registry path if present
            let file = record
                .file()
                .map(|file| {
                    if file.contains("/.cargo/") {
                        let mut file = file.split("/.cargo/").last().unwrap();
                        for _ in 0..3 {
                            if let Some(index) = file.find("/") {
                                file = &file[index + 1..];
                            } else {
                                return file;
                            }
                        }
                        file
                    } else {
                        file
                    }
                })
                .unwrap_or("?");

            writeln!(
                buf,
                "{} {} [{}:{}] {}",
                chrono::Utc::now(),
                style.value(record.level()),
                file,
                record.line().unwrap_or(0),
                record.args(),
            )
        })
        .init();
}

