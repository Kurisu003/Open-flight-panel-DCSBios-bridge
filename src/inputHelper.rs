#![allow(non_snake_case)]
#![allow(unused_parens)]

use hidapi::{ HidApi, HidDevice };
use std::collections::HashMap;
use std::io::{self, Write};

use std::sync::{LazyLock, Mutex, OnceLock, RwLock};
use std::time::Instant;
use std::{thread, time::Duration};

use crate::types::PFP_KEYCODES;

static PRESSED_BUTTONS: LazyLock<RwLock<[u8; 72]>> =
    LazyLock::new(|| RwLock::new([0; 72]));

fn format_inputs(input_raw: &[u8]) -> Vec<u8> {
    let raw_shortened: &[u8] = &input_raw[1..10];
    let mut out = Vec::with_capacity(raw_shortened.len() * 8);
    for &byte in raw_shortened {
        for bit in 0..8 {
            out.push(((byte >> bit) & 1) as u8);
        }
    }
    out
}

fn update_pressed_buttons(formatted_inputs: Vec<u8>) {
    let mut buttons = PRESSED_BUTTONS.write().expect("pressed_buttons poisoned");
    buttons.copy_from_slice(&formatted_inputs);
}

// returns the character of the button that is pressed
// if multiple buttons are pressed returns the one furthest up in list
// valid buttons are 0..9, A..Z, Space, /, .
pub fn get_button_pressed_char() -> String{ let guard = PRESSED_BUTTONS.read().expect("pressed_buttons poisoned");
    // numbers first
    for i in 28..=37 {
        if guard.get(i) == Some(&1) {
            let ch= char::from_digit((i as u32) - 28, 10).unwrap();
            return ch.to_string();
        }
    }
    if(guard.get(38)==Some(&1)){
        return String::from(".")
    }
    if(guard.get(39)==Some(&1)){
        return String::from("0")
    }
    for i in 41usize..=66 { if guard.get(i) == Some(&1) {
        // Map 41 -> 'A', 42 -> 'B', ..., 66 -> 'Z'
        let ch = (b'A' + (i as u8 - 41)) as char;
            return ch.to_string();
        }
    }
    if(guard.get(69)==Some(&1)){
        return String::from("/")
    }
    return String::from("");
}

pub fn is_button_pressed(button: &str) -> bool{
    let guard = PRESSED_BUTTONS.read().expect("pressed_buttons poisoned");

    let index = match PFP_KEYCODES.get(button) {
        Some(&i) => i as usize,
        None => return false, // unknown button
    };
    return guard.get(index).copied().unwrap_or(0) == 1;
}

// copy sent to keep original reference in main
pub fn poll_nonblocking(dev: &HidDevice) -> hidapi::HidResult<()> {
    dev.set_blocking_mode(false)?; // switch to non-blocking
    let mut buf = [0u8; 64];
    loop {
        match dev.read(&mut buf) {
            Ok(n) if n > 0 => {
                let report = &buf[..n];
                if report[0] == 1{
                    update_pressed_buttons(format_inputs(report));
                    // println!("{:?}", get_button_pressed_char());
                }
            }
            Ok(_) => {
                // no data right now
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => {
                // handle I/O error (disconnect, etc.)
                eprintln!("hid read error: {e}");
                break;
            }
        }
    }
    Ok(())
}