// Copyright (c) 2019 Zachary Nielsen
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::{convert::TryInto, string::ToString, cmp::PartialEq};
use structopt::StructOpt;

#[derive(PartialEq)]
enum ErrorCode {
    BaseConversionErr,
    TargetBaseErr,
    InputBaseErr,
}

impl std::fmt::Debug for ErrorCode {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match *self {
            ErrorCode::BaseConversionErr => "Base Conversion Error",
            ErrorCode::TargetBaseErr     => "Target Base Error",
            ErrorCode::InputBaseErr      => "Input Base Error",
        })
    }
}

fn main() -> Result<(), ErrorCode> {
    // Get args
    let opt = Opt::from_args();

    if opt.verbosity > 0 {
        println!("{:?}", opt);
    }

    //
    // Sort out the optional indexed argument
    //
    let mut to_bases: Vec<String>    = opt.to_bases.clone();
    let bases = get_bases(&opt, &mut to_bases);
    let from_base: u32 = bases.0;
    let from_num = bases.1;

    if to_bases.is_empty() {
        to_bases = vec![
            "2" .to_string(),
            "8" .to_string(),
            "10".to_string(),
            "16".to_string()
        ]
    }

    //
    // Convert input number to base 10
    //
    let num = convert_to_base_10(from_num, from_base, opt.sep_char)?;

    // Print conversions
    for target_base in to_bases {
        let custom_base = match u32::from_str_radix(&target_base, 10) {
            Ok (v) => v,
            Err(_) => {
                println!("Error with target base {}\nPlease provide target base is base 10.", target_base);
                return Err(ErrorCode::TargetBaseErr);
            },
        };
        let mut out_str = match as_string_base(&num, custom_base) {
            Ok(v)  => v,
            Err(e) => {
                println!("Error with custom base:\n\t{}", e);
                return Err(ErrorCode::InputBaseErr);
            },
        };

        if !opt.silent {
            if !opt.no_sep && opt.sep_length > 0 {
                // Pad string every opt.spacer_length characters
                // Need size-1/spacer_len additional slots in the string
                let mut insert_idx: i32 = out_str.len() as i32 - opt.sep_length as i32;
                while insert_idx > 0 {
                    let left  = String::from(&out_str[..(insert_idx as usize)]);
                    let right = String::from(&out_str[(insert_idx as usize)..]);
                    out_str = left;
                    out_str.push(opt.sep_char);
                    out_str.push_str(&right);
                    insert_idx -= opt.sep_length as i32;
                }
            }
            if !opt.bare {
                print!("Base {:02}: ", &custom_base);
            }
            println!("{}", out_str);
        }
    }
    return Ok(());
}

fn get_from_base(from_base: &str) -> Option<u32>
{
    match from_base {
        "b" => Some(2),
        "o" => Some(8),
        "d" => Some(10),
        "h" | "x" => Some(16),
        _   => None,
    }
}

fn get_bases(opt: &Opt, to_bases: &mut Vec<String>) -> (u32, Option<String>) {
    match get_from_base(opt.from_base_char.as_str()) {
        Some(v) => (v, opt.from_num.clone()),
        None => {
            // No base_char. Push from_num to the bases Vec, push base_char to from_num.
            if let Some(a_base) = &opt.from_num {
                to_bases.insert(0, a_base.clone());
            }
            // base_char wasn't provided, use the `-b` flag value as the base.
            (opt.from_base, Some(opt.from_base_char.clone()))
        },
    }
}

fn convert_to_base_10(from_num: Option<String>, from_base: u32, sep_char: char) -> Result<u128, ErrorCode> {
    let from_num = if let Some(num) = from_num {
        num.replace(sep_char, "")
    } else {
        println!("no number to convert was provided");
        return Err(ErrorCode::InputBaseErr);
    };

    match u128::from_str_radix(&from_num, from_base) {
        Ok(v)  => Ok(v),
        Err(_e) => {
            println!("Could not convert {} from base {}", from_num, from_base);
            return Err(ErrorCode::BaseConversionErr);
        },
    }
}

fn as_string_base(num: &u128, base: u32) -> Result<String, String>
{
    if base<2 || base>33 {
        Err(String::from("Invalid Base.  Base must be between 2 and 32 inclusive"))
    }
    else {
        let mut str_num = String::new();

        let mut tmp: u128 = *num;
        let mut count: u32 = 0;

        while tmp > 0 {
            let radix_mask: u128 = u128::from((base as u128).pow(count));
            let digit: u8 = match ((tmp / radix_mask) % u128::from(base)).try_into() {
                Ok(v)  => v,
                Err(_) => {
                    return Err(format!("Error while trying to convert to radix {}", base));
                },
            };

            let ch = if digit >= 10 {
                (b'A' + (digit-10)) as char
            }
            else {
                (b'0' + digit) as char
            };

            str_num = ch.to_string() + str_num.as_str();

            count += 1;
            tmp -= u128::from(digit) * radix_mask;
        }

        Ok(str_num)
    }
}


#[derive(StructOpt, Debug)]
#[structopt(name = "numconverter", about = "A CLI number conversion utility written in Rust")]
struct Opt {
    /// Pad the output with leading 0s
    #[structopt(short, long, default_value = "0")]
    pad: u8,

    /// Put a spacer every N characters
    #[structopt(short = "-l", long, default_value = "4")]
    sep_length: u32,

    /// Specify spacer char
    #[structopt(long, default_value = "_")]
    sep_char: char,

    /// Do not pad the output
    #[structopt(long)]
    no_sep: bool,

    /// Input Base
    ///
    /// base_char takes precedence over this setting
    #[structopt(short, long, default_value = "10")]
    from_base: u32,

    /// Do not print output (for use with clipboard)
    #[structopt(short, long)]
    silent: bool,

    /// Disable Pretty Print
    #[structopt(long)]
    bare: bool,

    /// Verbosity (more v's, more verbose)
    #[structopt(short, long, parse(from_occurrences))]
    verbosity: u8,

    /// Char representation of input base (b, o, d, or h) [optional]
    from_base_char: String,

    /// Number to convert
    from_num: Option<String>,

    /// Bases to convert to
    to_bases: Vec<String>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin() {
        assert_eq!(as_string_base(&4,   2).unwrap(), "100");
        assert_eq!(as_string_base(&12,  2).unwrap(), "1100");
        assert_eq!(as_string_base(&187, 2).unwrap(), "10111011");
        assert_eq!(as_string_base(&69,  2).unwrap(), "1000101");
    }

    #[test]
    fn test_oct() {
        assert_eq!(as_string_base(&4,   8).unwrap(), "4");
        assert_eq!(as_string_base(&12,  8).unwrap(), "14");
        assert_eq!(as_string_base(&187, 8).unwrap(), "273");
        assert_eq!(as_string_base(&69,  8).unwrap(), "105");
    }

    #[test]
    fn test_hex() {
        assert_eq!(as_string_base(&4,   16).unwrap(), "4");
        assert_eq!(as_string_base(&12,  16).unwrap(), "C");
        assert_eq!(as_string_base(&187, 16).unwrap(), "BB");
        assert_eq!(as_string_base(&69,  16).unwrap(), "45");
    }

    #[test]
    fn test_get_bases() {
        let mut opt = Opt{
            pad: 0,
            sep_length: 4,
            sep_char: '_',
            no_sep: false,
            from_base: 10,
            silent: false,
            bare: false,
            verbosity: 0,
            from_base_char: "b".to_owned(),
            from_num: Some("187".to_owned()),
            to_bases: Vec::new(),
        };

        let mut to_bases: Vec<String> = opt.to_bases.clone();
        let res = get_bases(&opt, &mut to_bases);
        assert_eq!(res.0, 2);
        assert_eq!(res.1, Some("187".to_owned()));
        assert!(to_bases.is_empty());

        opt.from_base_char = "80".to_owned();
        let res = get_bases(&opt, &mut to_bases);
        assert_eq!(res.0, 10);
        assert_eq!(res.1, Some("80".to_owned()));
        assert!(!to_bases.is_empty());
    }

    #[test]
    fn test_convert_to_base_10() {
        assert_eq!(convert_to_base_10(Some("10111011".to_owned()), 2, '_'), Ok(187));
        assert_eq!(convert_to_base_10(Some("273".to_owned()), 8, '_'), Ok(187));
        assert_eq!(convert_to_base_10(Some("187".to_owned()), 10, '_'), Ok(187));
        assert_eq!(convert_to_base_10(Some("BB".to_owned()), 16, '_'), Ok(187));
        assert_eq!(convert_to_base_10(None, 10, '_'), Err(ErrorCode::InputBaseErr));
    }
}
