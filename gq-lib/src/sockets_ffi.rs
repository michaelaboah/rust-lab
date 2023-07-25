use std::ffi::{c_char, c_void, CStr};
use std::sync::mpsc::{channel, Receiver};
use tungstenite::{connect, Message};
use url::Url;

#[no_mangle]
pub extern "C" fn create_socket(
    c_exchange: *const c_char,
    c_asset_class: *const c_char,
    c_data_type: *const c_char,
    c_symbol: *const c_char,
) -> *mut c_void {
    let exchange = unsafe { CStr::from_ptr(c_exchange).to_string_lossy().to_string() };
    let asset_class = unsafe { CStr::from_ptr(c_asset_class).to_string_lossy().to_string() };
    let data_type = unsafe { CStr::from_ptr(c_data_type).to_string_lossy().to_string() };
    let symbol = unsafe { CStr::from_ptr(c_symbol).to_string_lossy().to_string() };

    let (mut socket, _response) =
        connect(Url::parse("ws://localhost:5050/ws").unwrap()).expect("Can't connect");
    let message = format!(
        r#"{{"event":"subscribe", "channel":"{exchange}.{asset_class}.{data_type}.{symbol}"}}"#
    );
    // println!("From rust: {}", &message);
    // binance.spot.ohlcv.BTCUSDT
    socket
        .write_message(tungstenite::Message::Text(message))
        .unwrap();

    let (tx, rx) = channel();
    // Spawn a thread to handle the socket and send messages to the channel
    std::thread::spawn(move || {
        // TODO: Create and handle the socket here
        loop {
            if let Ok(msg) = socket.read_message() {
                if let Message::Text(t) = msg {
                    tx.send(t).unwrap();
                }
            } else {
                break;
            }
            // Receive a message from the socket and send it to the channel
            // let message = String::from("some message from socket");
            // sender.send(message).unwrap();
        }
    });
    // Return a pointer to the receiver channel
    Box::into_raw(Box::new(rx)) as *mut c_void
}

#[no_mangle]
pub extern "C" fn receive_message(receiver_ptr: *mut c_void) -> *const libc::c_char {
    let receiver = unsafe { &*(receiver_ptr as *const Receiver<String>) };
    // Receive a message from the channel
    match receiver.recv() {
        Ok(message) => {
            // Convert the Rust String to a C string and return a pointer to it
            let c_str = std::ffi::CString::new(message).unwrap();
            c_str.into_raw() as *const libc::c_char
        }
        Err(_) => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn destroy_socket(receiver_ptr: *mut c_void) {
    // Free the memory for the receiver channel
    let _ = unsafe { Box::from_raw(receiver_ptr as *mut Receiver<String>) };
}
