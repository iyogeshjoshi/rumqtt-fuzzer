extern crate rumqtt_fuzzer;
extern crate slog;

use rumqtt_fuzzer::{Fuzzer, MqHarness};

const LOCAL_BROKER: &'static str = "localhost:1883";
const BROKER: &'static str = "perf-mqtt-broker.atherengineering.in:5000";

fn main() {
	let pack = Fuzzer::make_packet(199);
	// Publish Subscribe callbacks test

	MqHarness::spawn_pub_sub_with_tls("test/big/qos1/stress", pack, 500, None, "big_publish_alot.log", BROKER);
	
	// Publish Ack test

	//MqHarness::spawn_pubacks_test("test/big/qos1/stress", pack, 100000, None, "big_publish_alot.log", BROKER);

	// Userdata publish test
	// let userdata = b"arandommqttuser:sekretpassword".to_vec();
	// let pack = Fuzzer::make_packet(50);
	// MqHarness::spawn_userdata_publish_test("test/big/qos1/stress", pack, 10000, None, "big_publish_alot.log", BROKER, userdata);
}
