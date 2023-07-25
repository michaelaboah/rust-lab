use crate::models::*;
use serde_json;
use std::ffi::{c_char, c_void, CStr};

#[no_mangle]
pub extern "C" fn create_csv(json_vec_ptr: *const c_char) {
    let json_str = unsafe { CStr::from_ptr(json_vec_ptr).to_string_lossy().to_string() };
    // println!("{json_str}");
    let json_list = serde_json::from_str::<Vec<serde_json::Value>>(&json_str).unwrap();
    // println!("{:#?}", json_list[0].as_str().unwrap().split(" ")[1]);
    // let mut wtr =
    //     csv::Writer::from_path(std::env::current_dir().unwrap().join("test.csv")).unwrap();

    // for json in json_list {
    //     let val = json.get("symbol").unwrap();
    //     wtr.serialize(val).unwrap();
    // }
    // wtr.flush().unwrap();
}
