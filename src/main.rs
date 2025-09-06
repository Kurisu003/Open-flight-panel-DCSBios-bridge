#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_parens)]
mod writeHelper;
mod dcsBiosHelper;
mod moduleDataProcessorHelper;
mod inputHelper;
mod searchModeHelper;
pub(crate) mod types;

use crate::types::{TextBlock};
use crate::writeHelper::{send_init_from_file, send_text_to_disp};
use crate::dcsBiosHelper::{read_stream, get_map};
use crate::moduleDataProcessorHelper::{get_A10C2_text, get_AH64D_text, get_AV8B_text, get_CH47F_text, get_module_name, handle_A10C2_input, handle_AH64D_input};
use crate::inputHelper::{is_button_pressed, poll_nonblocking};
use crate::searchModeHelper::{get_search_mode_disp};

use anyhow::{ anyhow, Context, Result };
use hidapi::{ HidApi, HidDevice };
use std::thread;
use std::collections::HashMap;
use std::time::{Duration, Instant};


const VID: u16 = 0x4098;
const PID: u16 = 0xbb35; // PFP 3N
// const PID: u16 = 0xbb37; // PFP 7
const WRITE_DELAY_SHORT: f32 = 0.005;
const WRITE_DELAY_LONG: f32 = 0.01;
const TOGGLE_DELAY: u64 = 100;
const THREAD_SLEEP: u64 = 10;
const INIT_PATH: &str = "output4.txt";
const MANUAL_BUTTON_MAPPING: bool = false;

pub fn find_device() -> Result<HidDevice> {
    let hid_api = HidApi::new().context("Failed to initialize HID API")?;

    // Prefer opening by product string; fall back to the first matching VID/PID.
    let dev_info = hid_api
        .device_list()
        .find(|d|
            d.vendor_id() == VID &&
            d.product_id() == PID &&
            d.product_string().unwrap_or("").contains("WINWING PFP-3N-CAPTAIN")
        )
        .or_else(|| {
            hid_api
                .device_list()
                .find(|d| d.vendor_id() == VID && d.product_id() == PID)
        })
        .ok_or_else(|| anyhow!("PFP_WRITER: No HID interfaces with VID={:04X} PID={:04X} found by hidapi", VID, PID))?;

    let path = dev_info.path();

    let device = hid_api
        .open_path(path)
        .with_context(|| format!("PFP_WRITER: Failed to open HID path {}", path.to_string_lossy()))?;

    Ok(device)
}


fn main() -> Result<()> {
    let write_device = find_device()?;
    let read_device = find_device()?;

    // Sends init package from file
    send_init_from_file(&write_device, INIT_PATH, WRITE_DELAY_SHORT);
    println!("PFP_WRITER: Sent init packets to {:04X}:{:04X} from {}", VID, PID, INIT_PATH);

    // let test = vec![TextBlock{ text: "Auvwxyz".to_string(), bg: "black".to_string(), fg: "white".to_string() }];
    // send_text_to_disp(&write_device, WRITE_DELAY_S,&test);

    // Spawn background thread for DCS_Bios
    thread::spawn(|| {
        if let Err(e) = read_stream() {
            eprintln!("read_stream error: {}", e);
        }
    });

    // Spawn background thread for reading device inputs
    thread::spawn(move || {
        if let Err(e) = poll_nonblocking(&read_device) {
            eprintln!("PFP_WRITER: HID reader error: {e}");
        }
    });

    // variables required for switching to search mode
    let mut search_mode = false;
    let mut last_toggle = Instant::now() - Duration::from_millis(TOGGLE_DELAY);
    loop {
        // msb = mode switch button
        let msb_pressed = is_button_pressed("MENU");
        if (msb_pressed
            && last_toggle.elapsed() >= Duration::from_millis(TOGGLE_DELAY)) {
                search_mode = !search_mode;
                last_toggle = Instant::now();
        }

        if(search_mode && !MANUAL_BUTTON_MAPPING){
            let mut res: Vec<TextBlock> = Vec::new();
            res.push(
                TextBlock {
                    text: String::from("Search Mode: "),
                    bg: (String::from("black")),
                    fg: (String::from("green"))
                }
            );

            let search_mode_disp = get_search_mode_disp();
            send_text_to_disp(&write_device, WRITE_DELAY_LONG, &search_mode_disp);
        }


        // normal display mode
        if(!search_mode){
            let map_arc = get_map(); // clone Arc so it lives long enough
            let snapshot: HashMap<u16, [u8; 2]> = {
                let guard = match map_arc.lock() {
                    Ok(g) => g,
                    Err(p) => p.into_inner(),
                };
                guard.clone() // clones the whole HashMap
            };

            // yes, I know this gets called every 10ms
            // yes, I know its not performant
            let module_name = get_module_name(&snapshot);
            // if(!module_name.starts_with("?")){
            //     println!("Module: {:}", module_name);
            // }

            let mut res: Vec<TextBlock> = Vec::new();
            if module_name.starts_with("A-10C_2"){
                res = get_A10C2_text(&snapshot);
                if (!MANUAL_BUTTON_MAPPING){
                    handle_A10C2_input()
                };
            }
            else if(module_name.starts_with("AV8B")){
                res = get_AV8B_text(&snapshot);
            }
            else if(module_name.starts_with("CH-47F")){
                res = get_CH47F_text(&snapshot);
            }
            else if(module_name.starts_with("AH-64D_BLK_II")){
                res = get_AH64D_text(&snapshot);
                if (!MANUAL_BUTTON_MAPPING){
                    handle_AH64D_input(&snapshot)
                };
            }

            // send_text_to_disp(&write_device, 0.01, &res);
            send_text_to_disp(&write_device, WRITE_DELAY_LONG,&res);
        }

        thread::sleep(Duration::from_millis(THREAD_SLEEP));
    }

    // Ok(())
}
