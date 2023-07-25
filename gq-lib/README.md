# Rust Dynamic Library for WebSocket Connections

This library provides a Rust implementation of WebSocket connections that can be targeted by other programming languages, such as Python, Node.js, and C#. This library is designed to be compiled into a dynamic library, making it easy to include in other projects.

Features

    Create a WebSocket connection with the specified exchange, asset class, data type, and symbol
    Receive messages from the WebSocket channel
    Destroy the WebSocket connection and free memory

## _Dependencies_

This library depends on the following crates:

    Tungstenite for WebSocket support. native-tls is enabled
    Url for URL parsing
    Libc for C interop

## _Usage_

    Clone the repository and navigate to the root directory
    Build the dynamic library using cargo build --release
    Link the dynamic library to your project in the desired language
    Use the exposed functions to create a WebSocket connection, receive messages, and destroy the connection

## _Notes_

**Dynamic libary is specific to targeted OS:**

- Linux: libgq_rust.so
- MacOs: libgq_rust.dylib
- Win32: libgq_rust.dll
