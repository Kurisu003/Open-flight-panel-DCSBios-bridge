#![allow(non_snake_case)]
use anyhow::{ Context, Result };
use hidapi::{ HidDevice };
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::thread::{self, sleep};
use std::time::Duration;
use maplit::hashmap;

use crate::types::TextBlock;

const PAD_TO_LEN: Option<usize> = None; // e.g. Some(64) if fixed-size reports are required
const PREPEND_REPORT_ID_00: bool = false; // set true if reports must start with 0x00


fn fix_text_length(mut block: Vec<TextBlock>) -> Vec<TextBlock>{
    let max_len = 24*14;
    let total_len: usize = block.iter().map(|element| element.text.len()).sum();

    // Cuts off at the end
    if total_len > max_len{
        let mut remaining: usize = max_len;
        for(_, element) in block.iter_mut().enumerate(){
            let s: &str = &element.text;

            if remaining <= 0 {
                element.text = String::from("");
                continue;
            }

            if s.len() > remaining{
                element.text = s.chars().take(remaining).collect();
                remaining = 0;
            }
            else{
                remaining -= s.len();
            }
        }
    }
    // Pads the end if too short
    else if total_len < max_len{
        let spaces_needed = max_len - total_len;
        if let Some(last) = block.last_mut(){
            last.text.push_str(&" ".repeat(spaces_needed));
        }
    }
    return block;
}

fn get_payload_from_blocks(blocks: &Vec<TextBlock>) -> Vec<u8>{
    let fg_lookup = hashmap! {
        String::from("orange") => 0,
        String::from("white") => 1,
        String::from("cyan") => 2,
        String::from("green") => 3,
        String::from("magenta") => 4,
        String::from("red") => 5,
        String::from("yellow") => 6
    };

    let bg_lookup = hashmap!{
        String::from("black") => 0,
        String::from("green") => 1,
        String::from("gray") => 2,
        String::from("orange") => 3,
        String::from("purple") => 4
    };

    // pass cloned version of referenced block to fix_text_length
    // and then continue working with that
    let fixed_block: Vec<TextBlock> = fix_text_length(blocks.clone());

    let mut payload: Vec<u8> = Vec::new();
    for element in fixed_block{
        let mut fg: &String = &element.fg.clone();
        let mut bg: &String = &element.bg.clone();

        let white: String = String::from("white");
        let black: String = String::from("black");

        fg = if fg_lookup.contains_key(fg) { fg } else { &white };
        bg = if bg_lookup.contains_key(bg) { bg } else { &black };

        let prefix_byte: u8 = 0x21 + 0x21 * fg_lookup[fg] + 0xC * bg_lookup[bg];

        let text_bytes: Vec<u8> = element.text
            .chars()
            .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
            .collect();

        let mut block_playload: Vec<u8> = Vec::new();
        for byte in text_bytes.iter(){
            block_playload.extend([prefix_byte, 0x00, *byte])
        }

        payload.extend(block_playload)
    }

    return payload
}

fn text_to_hex_packet(blocks: &Vec<TextBlock>) -> Vec<Vec<u8>>{
    let mut packets: Vec<Vec<u8>> = Vec::new();
    let payload: Vec<u8> = get_payload_from_blocks(blocks);

    let mut i = 0;
    while i < payload.len(){
        let mut chunk: Vec<u8> = payload[i..(i + 63).min(payload.len())].to_vec();
        i += 63;
        let remaining: usize = 63usize.saturating_sub(chunk.len());
        let triplets_remaining: usize = remaining / 3;

        for _ in 0..triplets_remaining{
            chunk.extend_from_slice(&[0x42, 0x00, 0x20]);
        }
        chunk.resize(63,0x00);

        let mut packet: Vec<u8> = Vec::with_capacity(1 + chunk.len());
        packet.push(0xF2);
        packet.extend_from_slice(&chunk);

        packets.push(packet);
    }
    return packets;
}

pub fn send_text_to_disp(device: &HidDevice,write_delay: f32, blocks: &Vec<TextBlock>) {
    let hex_packets = text_to_hex_packet(blocks);
    for (_, element )in hex_packets.iter().enumerate(){
        #[warn(unused_must_use)]
        let _ = device.write(&element);
        thread::sleep(Duration::from_secs_f32(write_delay));

    }
}

pub fn send_init_from_file(device: &HidDevice, init_path: &str, delay_secs: f32) {
    let file = match File::open(init_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open '{}': {}", init_path, e);
            return;
        }
    };

    let reader = BufReader::new(file);
    let delay = Duration::from_secs_f32(delay_secs.max(0.0));

    for (lineno, line_res) in reader.lines().enumerate() {
        let line_no = lineno + 1;
        let line = match line_res {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Line {}: read error: {}", line_no, e);
                continue;
            }
        };

        // Strip comments and whitespace.
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue; // skip blank/comment-only lines
        }

        // Remove all whitespace to allow "AA BB CC" or "AABBCC" styles.
        let hex: String = trimmed.split_whitespace().collect();

        if hex.len() % 2 != 0 {
            eprintln!("Line {}: odd number of hex digits: '{}'", line_no, trimmed);
            continue;
        }

        // Convert hex string to bytes.
        let mut bytes = Vec::with_capacity(hex.len() / 2);
        let mut bad_pair = false;
        for i in (0..hex.len()).step_by(2) {
            match u8::from_str_radix(&hex[i..i + 2], 16) {
                Ok(b) => bytes.push(b),
                Err(e) => {
                    eprintln!(
                        "Line {}: invalid hex pair '{}': {}",
                        line_no,
                        &hex[i..i + 2],
                        e
                    );
                    bad_pair = true;
                    break;
                }
            }
        }
        if bad_pair {
            continue;
        }

        // Send to the HID device.
        if let Err(e) = device.write(&bytes) {
            eprintln!("Line {}: HID write failed: {}", line_no, e);
        }

        // Required delay after every send.
        sleep(delay);
    }
}