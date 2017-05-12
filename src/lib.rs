#![feature(integer_atomics)] 

//! This library aims to create a test harness for mqttclients, which tries to simulate arbitrary packets
//! along side allowing user defined topic for publish and subscribes coupled with logging of messages on user defined callbacks
//! to a log file using `slog-rs` library.

extern crate rand;
extern crate rumqtt;
extern crate mqtt;
#[macro_use]
extern crate slog;
extern crate slog_stream;
extern crate slog_stdlog;
#[macro_use]
extern crate log;
use std::io;

pub mod generator;
pub use generator::{Fuzzer, MqHarness};

#[test]
fn it_works() {

}
