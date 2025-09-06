// src/stream.rs
#![allow(unused_parens)]


use std::io::{ErrorKind, Read, Result, Write};
use std::net::TcpStream;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use hex;
use std::io::{self};
use std::thread::sleep;


pub type SharedMap = Arc<Mutex<HashMap<u16, [u8; 2]>>>;
const BUTTON_SLEEP_DELAY: u64 = 100;
const STREAM_TIMEOUT: u64 = 500;

/// Public, static, thread-safe hash map (u8 → u8)
pub static GLOBAL_MAP: Lazy<SharedMap> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// Public getter (returns an Arc clone)
pub fn get_map() -> SharedMap {
    GLOBAL_MAP.clone()
}

fn update_vals(data: &[u8]) {
    if data.len() < 8 { return; }
    let mut pos = 4; // skip 8-byte stream header

    let map = get_map();
    let mut m = map.lock().unwrap();

    while pos + 4 <= data.len() {
        // Header for this block
        let start = u16::from_le_bytes([data[pos],     data[pos + 1]]);
        let len   = u16::from_le_bytes([data[pos + 2], data[pos + 3]]); // FIXED
        pos += 4;

        let byte_len = len as usize;
        if pos + byte_len > data.len() {
            // Caller must supply remainder in a subsequent call (see B)
            break;
        }

        // DCS-BIOS payload is a sequence of 2-byte words; addresses advance by 2
        let mut addr = start;
        let payload = &data[pos .. pos + byte_len];
        for w in payload.chunks_exact(2) {
            m.insert(addr, [w[0], w[1]]);
            addr = addr.wrapping_add(2);
        }
        pos += byte_len;
    }
}

pub fn read_stream() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:7778")?;
    // Block up to 200 ms per read; if no data arrives, return TimedOut.
    stream.set_read_timeout(Some(Duration::from_millis(STREAM_TIMEOUT)))?;

    let mut buffer = [0u8; 4096];
    let mut is_first_time = true;

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                eprintln!("Connection closed by peer");
                return Ok(());
            }
            Ok(n) => {
                // Process the received bytes
                if (is_first_time){
                    println!("\nConnection OK");
                    is_first_time = false;
                }
                update_vals(&buffer[..n]);
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => {
                // No data within the timeout — avoid a tight spin loop.
                sleep(Duration::from_millis(5));
                continue;
            }
            Err(e) => {
                eprintln!("Connection closed");

                // Propagate unexpected I/O errors to the caller.
                return Err(e);
            }
        }
    }
}

pub fn send_button_press(button: &str){
    let _ = send_message_to_dcsbios(&button);
    sleep(Duration::from_millis(BUTTON_SLEEP_DELAY));
    let _ = send_message_to_dcsbios(&button);
}

pub fn send_button_state_press(state_1: &str, state_2: &str){
    let _ = send_message_to_dcsbios(&state_1);
    sleep(Duration::from_millis(BUTTON_SLEEP_DELAY));
    let _ = send_message_to_dcsbios(&state_2);
}

pub fn send_message_to_dcsbios(message: &str) -> io::Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", 7778))?;
    stream.write_all(message.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}