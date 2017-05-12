use std::path::Path;
use std::fs;
use std::io;

use rand;
use rand::Rng;
use rumqtt::{MqttOptions, MqttCallback, MqttClient};
use std::fs::OpenOptions;

use slog_stdlog::set_logger;
use slog::DrainExt;
use slog_stream;
use slog;

use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use mqtt::QualityOfService;

pub struct Fuzzer;
pub struct MqHarness;

pub fn setup_logging(file_name: &str) {
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
}

impl MqHarness {
    pub fn spawn_pub_sub_with_pack(topic: &str,
                                   pack: Vec<u8>,
                                   npack: u64,
                                   log_path: Option<&str>,
                                   file_name: &str,
                                   broker: &str) {
        let count = Arc::new(AtomicU64::new(0));
        let recv_cnt = count.clone();

        setup_logging(file_name);

        let client_options = MqttOptions::new()
                                        .set_keep_alive(5)
                                        .set_reconnect(5).
                                        set_storepack_sz(500)
                                        .set_broker(broker);

        info!("{:?}\n\n", "RUMQTT-FUZZER Logs");

        let callback = move |msg| {
            count.fetch_add(1, Ordering::SeqCst);
            warn!("Packet no processed: {:?}", count);
        };

        let mq_cb = MqttCallback::new().on_message(callback);
        let mut mq_client = MqttClient::start(client_options, Some(mq_cb)).expect("Couldn't start");
        let sub = vec![(topic, QualityOfService::Level1)];

        let mut send_cnt = 0;

        mq_client.subscribe(sub).expect("Subscription failure");
        for _ in 0..npack {
            match mq_client.publish(topic, QualityOfService::Level1, pack.clone()) {
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

        pub fn spawn_pub_sub_with_tls(topic: &str,
                                   pack: Vec<u8>,
                                   npack: u64,
                                   log_path: Option<&str>,
                                   file_name: &str,
                                   broker: &str) {
        let count = Arc::new(AtomicU64::new(0));
        let recv_cnt = count.clone();

        setup_logging(file_name);

        let client_options = MqttOptions::new()
                                        .set_keep_alive(5)
                                        .set_reconnect(5).
                                        set_storepack_sz(200)
                                        .set_ca("utils/ca-chain.cert.pem")
        .set_client_cert("utils/s340.cert.pem", "utils/s340.key.pem")
                                        .set_broker(broker);

        info!("{:?}\n\n", "RUMQTT-FUZZER Logs");

        let callback = move |msg| {
            count.fetch_add(1, Ordering::SeqCst);
            warn!("Packet no processed: {:?}", count);
        };

        let mq_cb = MqttCallback::new().on_message(callback);
        let mut mq_client = MqttClient::start(client_options, Some(mq_cb)).expect("Couldn't start");
        let sub = vec![(topic, QualityOfService::Level1)];

        let mut send_cnt = 0;

        mq_client.subscribe(sub).expect("Subscription failure");
        for _ in 0..npack {
            match mq_client.publish(topic, QualityOfService::Level1, pack.clone()) {
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
        let count = Arc::new(AtomicU64::new(0));
        let recv_cnt = count.clone();
        setup_logging(file_name);

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
        let sub = vec![(topic, QualityOfService::Level1)];
        let mut send_cnt = 0;
        for _ in 0..npack {
            match mq_client.publish(topic, QualityOfService::Level1, pack.clone()) {
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

    pub fn spawn_userdata_publish_test(topic: &str,
                                       pack: Vec<u8>,
                                       npack: u64,
                                       log_path: Option<&str>,
                                       file_name: &str, 
                                       broker: &str,
                                       userdata: Vec<u8>) {
        let count = Arc::new(AtomicU64::new(0));
        let recv_cnt = count.clone();
        setup_logging(file_name);

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
        let sub = vec![(topic, QualityOfService::Level1)];
        let mut send_cnt = 0;
        for _ in 0..npack {
            match mq_client.userdata_publish(topic, QualityOfService::Level1, pack.clone(), userdata.clone()) {
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
