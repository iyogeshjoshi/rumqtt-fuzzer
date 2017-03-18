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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

pub struct Fuzzer;
pub struct MqHarness;


impl MqHarness {
	pub fn spawn_pub_sub_with_pack(topic: &str,
							   pack: Vec<u8>,
							   npack: u64,
							   log_path: Option<&str>,
							   file_name: &str, 
							   broker: &str) {
		let count = Arc::new(AtomicUsize::new(0));
		let recv_cnt = count.clone();
		let log_path = Path::new(file_name);
		let file = OpenOptions::new()
	        .create(true)
	        .write(true)
	        .truncate(true)
			.open(log_path).unwrap();

		// Set slog to use MqHarness as the Format implementation
		let drain = slog_stream::stream(file, MqHarness).fuse();
		let drain = slog::level_filter(slog::Level::Info, drain);
		let logger = slog::Logger::root(drain, o!());
		set_logger(logger).unwrap();

		let client_options = MqttOptions::new()
	                                    .set_keep_alive(5)
	                                    .set_reconnect(5)
	                                    .set_broker(broker);

        info!("{:?}\n\n", "RUMQTT-FUZZER Logs");

	    let callback = move |msg| {
	    	count.fetch_add(1, Ordering::SeqCst);

	    	warn!("Packet no processed: {:?}", count);
	    };

	    let mq_cb = MqttCallback::new().on_message(callback);
	    let mut mq_client = MqttClient::start(client_options, Some(mq_cb)).expect("Couldn't start");
	    let sub = vec![(topic, QoS::AtLeastOnce)];

	    let mut send_cnt = 0;

	    mq_client.subscribe(sub).expect("Subscription failure");
	    for _ in 0..npack {
	    	match mq_client.publish(topic, QoS::AtLeastOnce, pack.clone()) {
	    		Ok(_) => {send_cnt+=1;}
	    		Err(e) => {
	    			error!("{:?}", e);
	    		}
	    	}
	    }

	    // wait till all callbacks have done execution
	   	thread::sleep_ms(60000);

	    info!("\n\n[Packet Statistics]\n\nSent Count: {:?}\nReceived Count: {:?}", send_cnt, recv_cnt.load(Ordering::SeqCst));
	    assert_eq!(send_cnt, recv_cnt.load(Ordering::SeqCst));
	}

	/// Tests for publish acks from publish messages sent from client
	pub fn spawn_pubacks_test(topic: &str,
							   pack: Vec<u8>,
							   npack: u64,
							   log_path: Option<&str>,
							   file_name: &str, 
							   broker: &str) {
		let count = Arc::new(AtomicUsize::new(0));
		let recv_cnt = count.clone();
		let log_path = Path::new(file_name);
		let file = OpenOptions::new()
	        .create(true)
	        .write(true)
	        .truncate(true)
			.open(log_path).unwrap();
		let drain = slog_stream::stream(file, MqHarness).fuse();
		let drain = slog::level_filter(slog::Level::Info, drain);
		let logger = slog::Logger::root(drain, o!());
		set_logger(logger).unwrap();

		let client_options = MqttOptions::new()
	                                    .set_keep_alive(5)
	                                    .set_reconnect(5)
	                                    .set_broker(broker);

	    info!("{:?}\n\n", "RUMQTT-FUZZER Logs");

	    let callback = move |msg| {
	    	count.fetch_add(1, Ordering::SeqCst);
	    	warn!("ACK processed: {:?}", count);
	    };

	    let mq_cb = MqttCallback::new().on_publish(callback);
	    let mut mq_client = MqttClient::start(client_options, Some(mq_cb)).expect("Couldn't start");

		let topics = vec![("test/basic", QoS::AtMostOnce)];
    	mq_client.subscribe(topics).expect("Subcription failure");
	    
		let mut send_cnt = 0;
	    for _ in 0..npack {
	    	match mq_client.publish(topic, QoS::AtLeastOnce, pack.clone()) {
	    		Ok(_) => {send_cnt+=1;}
	    		Err(e) => {
	    			error!("{:?}", e);
	    		}
	    	}
	    }
	    // wait till all callbacks have done execution
	   	thread::sleep_ms(60000);

	   	info!("\n\n[Packet Statistics]\n\nSent Count: {:?}\nReceived Count: {:?}", send_cnt, recv_cnt.load(Ordering::SeqCst));
	    assert_eq!(send_cnt, recv_cnt.load(Ordering::SeqCst));
	}
}

impl Fuzzer {
	pub fn make_packet(kbs: usize) -> Vec<u8> {
		let big_payload: String = rand::thread_rng().gen_ascii_chars()
            										.take(kbs * 1024)
            										.collect();
		println!("{:?}", big_payload.len());
        big_payload.into_bytes()
	}
}

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
