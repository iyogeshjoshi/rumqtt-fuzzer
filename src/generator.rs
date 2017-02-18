use std::path::Path;
use std::fs;
use std::io;

use rand;
use rand::Rng;
use rumqtt::{MqttOptions, MqttCallback, QoS, MqttClient};
use std::fs::OpenOptions;

use slog_stdlog::set_logger;
use slog::DrainExt;
use slog_stream;
use slog;

pub struct Fuzzer;
pub struct MqHarness;

impl slog_stream::Format for MqHarness {
	fn format(&self,
              io: &mut io::Write,
              rinfo: &slog::Record,
              _logger_values: &slog::OwnedKeyValueList)
              -> io::Result<()> {
        let msg = format!("{} - {}\n", rinfo.level(), rinfo.msg());
        let _ = try!(io.write_all(msg.as_bytes()));
        Ok(())
    }
}

impl MqHarness {
	pub fn spawn_pub_sub_with_pack(topic: &str,
							   pack: Vec<u8>,
							   npack: u64,
							   log_path: Option<&str>,
							   file_name: &str, 
							   broker: &str) {
		let log_path = Path::new(file_name);
		let file = OpenOptions::new()
	        .create(true)
	        .write(true)
	        .truncate(true)
			.open(log_path).unwrap();

		// Set slog to use, MqHarness as the Format implemenatation.
		let drain = slog_stream::stream(file, MqHarness).fuse();
		let logger = slog::Logger::root(drain, o!());
		set_logger(logger).unwrap();

		let client_options = MqttOptions::new()
	                                    .set_keep_alive(5)
	                                    .set_reconnect(5)
	                                    .set_broker(broker);

	    let callback = move |msg| {
	    	info!("{:?}", msg);
	    };

	    let mq_cb = MqttCallback::new().on_message(callback);
	    let mut mq_client = MqttClient::start(client_options, Some(mq_cb)).expect("Couldn't start");
	    let sub = vec![(topic, QoS::Level1)];

	    // by default we will send Qos2 packets
	    mq_client.subscribe(sub).expect("Subscription failure");
	    for _ in 0..npack {
	    	mq_client.publish(topic, QoS::Level1, pack.clone()).unwrap();
	    }
	}
}

impl Fuzzer {
	pub fn make_packet(kbs: usize) -> Vec<u8> {
		let big_payload: String = rand::thread_rng().gen_ascii_chars()
            										.take(kbs * 1024)
            										.collect();
        big_payload.into_bytes()
	}
}