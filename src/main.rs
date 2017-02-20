extern crate rumqtt_fuzzer;
extern crate slog;

use rumqtt_fuzzer::{Fuzzer, MqHarness};

const LOCAL_BROKER: &'static str = "localhost:1883";
const BROKER: &'static str = "dev-mqtt-broker.atherengineering.in:1883";
/*

Goals of the mqtt_fuzzer -

1) Provide a unified interface for generating n packets, with user defined topics
2) Give way to user to also log files (we are using slog)
3) Give detailed reports of memory usage of all functions and callbacks invoked [Need discussion]
4) Spawn multiple clients from a single 

Implementation thoughts: It can be a cmd line tool with docopt style args or just a simple binary with predefined configs for initial use cases

*/

fn main() {
	let pack = Fuzzer::make_packet(1);
	/*MqHarness::spawn_pub_sub_with_pack("test/big/qos1/stress", pack, 500, None, "big_publish_alot.log", BROKER);*/
	MqHarness::spawn_pubacks_test("test/big/qos1/stress", pack, 500, None, "big_publish_alot.log", BROKER);
}
