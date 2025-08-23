#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::{env, io, path::{Path, PathBuf}, sync::{Mutex, OnceLock}};

use phf::phf_map;

use crate::{inputHelper::{get_button_pressed_char, is_button_pressed}, types::{rwr_info_lut, RwrInfo, TextBlock}};

static SEARCH_STRING: OnceLock<Mutex<String>> = OnceLock::new();

fn get_search_string() -> &'static Mutex<String> {
    SEARCH_STRING.get_or_init(|| Mutex::new(String::new()))
}

fn search_string_to_lookup_code(search_string: &str) -> String{
    match search_string{
        "2"   | "SA2"                   => "SA-2".to_string(),
        "3"   | "SA3"                   => "SA-3".to_string(),
        "5"   | "SA5"                   => "SA-5".to_string(),
        "6"   | "SA6"                   => "SA-6".to_string(),
        "8"   | "SA8"                   => "SA-8".to_string(),
        "9"   | "SA9"                   => "SA-9".to_string(),
        "10"  | "SA10"  | "BB" | "CS"   => "SA-10".to_string(),
        "11"  | "SA11"  | "SD"          => "SA-11".to_string(),
        "13"  | "SA13"                  => "SA-13".to_string(),
        "15"  | "SA15"                  => "SA-15".to_string(),
        "18"  | "SA18"                  => "SA-18".to_string(),
        "19"  | "SA19"  | "S6"          => "SA-19".to_string(),
        "24"  | "SA24"                  => "SA-24".to_string(),
        "23"  | "ZSU23"                 => "ZSU-23".to_string(),
        "234" | "ZSU234"| "A"           => "ZSU-23-4".to_string(),
        "572" | "ZSU572"                => "ZSU-57-2".to_string(),
        "GEPARD" | "GPRD" | "GE" | "GEP"=> "Gepard".to_string(),
        "M163" | "163" | "M1"           => "M163".to_string(),
        "M1097" | "1097" | "M10"        => "M1097".to_string(),
        "MIM23" | "MIM" | "MIM2" | "MI" => "MIM-23".to_string(),

        _ => search_string.to_string(),
    }
}

pub fn get_search_mode_disp()-> Vec<TextBlock>{
    // Constructing searchstring
    let mut search_string = get_search_string().lock().unwrap();

    let mut res: Vec<TextBlock> = Vec::new();

    if(is_button_pressed("CLR")){
        search_string.clear();
    }
    if(is_button_pressed("DEL")){
        search_string.pop();
    }
    let pressed_button = get_button_pressed_char();
    if(pressed_button != ""){
        search_string.push_str(&pressed_button);
    }

    let rwr_code = search_string_to_lookup_code(&search_string);
    let rwr_info = rwr_info_lut(&rwr_code);

    // Just formatting shenanigans
    let abrv_string = String::from("ABRV: ") + &rwr_info.abrv + &" ".repeat(24-6-rwr_info.abrv.len()).as_str();
    let nato_string = String::from("NATO: ") + &rwr_info.nato_name + &" ".repeat(24-6-rwr_info.nato_name.len()).as_str();
    let rwr_string = String::from("RWR: ") + &rwr_info.rwr_code + &" ".repeat(24-5-rwr_info.rwr_code.len()).as_str();
    let rng_nm_string = String::from("RNG NM: ") + &rwr_info.rng_nm + &" ".repeat(24-8-rwr_info.rng_nm.len()).as_str();
    let alt_ft_string = String::from("ALT FT: ") + &rwr_info.alt_ft + &" ".repeat(24-8-rwr_info.alt_ft.len()).as_str();
    let flr_string = String::from("FLARE: ") + &rwr_info.flr + &" ".repeat(24-7-rwr_info.flr.len()).as_str();
    let chf_string = String::from("CHAFF: ") + &rwr_info.chf + &" ".repeat(24-7-rwr_info.chf.len()).as_str();
    let ecm_string = String::from("ECM: ") + &rwr_info.ecm + &" ".repeat(24-5-rwr_info.ecm.len()).as_str();
    let lock_time_string = String::from("LOCK TIME: ") + &rwr_info.lock_time + &" ".repeat(24-11-rwr_info.lock_time.len()).as_str();
    let guidance_string = String::from("GUIDANCE: ") + &rwr_info.guidance + &" ".repeat(24-10-rwr_info.guidance.len()).as_str();

    let final_string = abrv_string + &nato_string + &rwr_string + &rng_nm_string + &alt_ft_string + &flr_string + &chf_string + &ecm_string + &lock_time_string +&guidance_string;

    res.push(
        TextBlock {
            text: String::from("SEARCH STRING: ") + search_string.as_str() + " ".repeat(9-search_string.len()).as_str(),
            bg: (String::from("black")),
            fg: (String::from("green"))
        }
    );

    res.push(
        TextBlock {
            text: final_string,
            bg: (String::from("black")),
            fg: (String::from("green"))
        }
    );


    return res;
}