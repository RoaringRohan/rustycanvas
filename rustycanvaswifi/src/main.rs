#![no_std]
#![no_main]

extern crate alloc;
use core::net::Ipv4Addr;
use heapless::String;

use blocking_network_stack::Stack;
use embedded_io::*;
use esp_alloc::{self as _, MemoryCapability};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    interrupt::software::SoftwareInterruptControl,
    main,
    rng::Rng,
    time::{self, Duration},
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::wifi::{ClientConfig, Config as WifiConfig, ModeConfig, ScanConfig};
use serde::Deserialize;
use smoltcp::{
    iface::{SocketSet, SocketStorage},
    wire::{DhcpOption, IpAddress},
};

const SSID: &str = "shortnet";
const PASSWORD: &str = "dictionary";

const COLOR_LEN: usize = 7; // "#RRGGBB"
const RESP_BUF_LEN: usize = 8192;

#[derive(Debug, Deserialize)]
pub struct Canvas {
    pub width: u8,
    pub height: u8,
    pub pixels: [[String<COLOR_LEN>; 32]; 16],
}

#[main]
fn main() -> ! {
    const HEAP_SIZE: usize = 96 * 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            &raw mut HEAP as *mut u8,
            HEAP_SIZE,
            MemoryCapability::Internal.into(),
        ))
    };

    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

    let radio = esp_radio::init().unwrap();

    let (mut controller, interfaces) =
        esp_radio::wifi::new(&radio, peripherals.WIFI, WifiConfig::default()).unwrap();

    let mut device = interfaces.sta;

    let iface = create_interface(&mut device);

    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = smoltcp::socket::dhcpv4::Socket::new();
    dhcp_socket.set_outgoing_options(&[DhcpOption {
        kind: 12,
        data: b"esp-radio",
    }]);
    socket_set.add(dhcp_socket);

    let rng = Rng::new();
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let stack = Stack::new(iface, device, socket_set, now, rng.random());

    controller
        .set_power_saving(esp_radio::wifi::PowerSaveMode::None)
        .unwrap();

    let client_cfg = ClientConfig::default()
        .with_ssid(SSID.into())
        .with_password(PASSWORD.into());

    let mode_cfg = ModeConfig::Client(client_cfg);

    let res = controller.set_config(&mode_cfg);

    println!("wifi_set_configuration returned {:?}", res);

    controller.start().unwrap();
    println!("is wifi started: {:?}", controller.is_started());

    println!("Start Wifi Scan");
    let scan_config = ScanConfig::default().with_max(10);
    let res = controller.scan_with_config(scan_config).unwrap();
    let mut found_network = false;
    for ap in res {
        let ssid_str = ap.ssid.as_str();
        println!(
            "Found AP: SSID={}, Channel={}, Signal={}",
            ssid_str, ap.channel, ap.signal_strength
        );
        if ssid_str == SSID {
            found_network = true;
            println!("  -> This is our target network! Auth={:?}", ap.auth_method);
        }
    }
    if !found_network {
        println!("WARNING: Target SSID '{}' not found in scan results!", SSID);
    }

    println!("{:?}", controller.capabilities());
    println!("wifi_connect {:?}", controller.connect());

    // wait to get connected
    println!("Wait to get connected");
    let connect_timeout = time::Instant::now() + Duration::from_secs(30);
    loop {
        match controller.is_connected() {
            Ok(true) => {
                println!("Connected!");
                break;
            }
            Ok(false) => {
                if time::Instant::now() > connect_timeout {
                    println!("ERROR: Connection timeout after 30 seconds");
                    loop {}
                }
            }
            Err(err) => {
                println!("Connection error: {:?}", err);
                loop {}
            }
        }
    }
    println!("Connection status: {:?}", controller.is_connected());

    // wait for getting an ip address
    println!("Wait to get an ip address");
    loop {
        stack.work();

        if stack.is_iface_up() {
            println!("got ip {:?}", stack.get_ip_info());
            break;
        }
    }

    println!("Start busy loop on main");

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut socket = stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {
        println!("Making HTTP request to 192.168.2.169:8080/canvas");
        socket.work();

        println!("Opening socket...");
        match socket.open(IpAddress::Ipv4(Ipv4Addr::new(192, 168, 2, 169)), 8080) {
            Ok(_) => println!("Socket opened"),
            Err(e) => {
                println!("Failed to open socket: {:?}", e);
                continue;
            }
        }

        println!("Sending HTTP request...");
        socket
            .write(b"GET /canvas HTTP/1.0\r\nHost: 192.168.2.169\r\n\r\n")
            .unwrap();
        socket.flush().unwrap();
        println!("Request sent");

        // Buffer for full HTTP response
        let mut response_buf = [0u8; RESP_BUF_LEN];
        let mut response_len = 0;

        let deadline = time::Instant::now() + Duration::from_secs(20);
        let mut buffer = [0u8; 512];

        println!("Reading response...");
        loop {
            match socket.read(&mut buffer) {
                Ok(len) => {
                    if len > 0 {
                        println!("Received {} bytes", len);
                        // Copy incoming bytes into response_buf
                        for &b in &buffer[..len] {
                            if response_len < response_buf.len() {
                                response_buf[response_len] = b;
                                response_len += 1;
                            }
                        }
                    }
                }
                Err(_) => break,
            }

            if time::Instant::now() > deadline {
                println!("Timeout after receiving {} bytes", response_len);
                break;
            }
        }

        println!("Total received: {} bytes", response_len);
        let full = &response_buf[..response_len];

        match core::str::from_utf8(full) {
            Ok(full_str) => {
                println!("Response as string (first 200 chars):");
                let preview = if full_str.len() > 200 {
                    &full_str[..200]
                } else {
                    full_str
                };
                println!("{}", preview);

                // Find beginning of JSON body
                if let Some(json_start) = full_str.find("\r\n\r\n") {
                    let json_str = &full_str[json_start + 4..];
                    println!("Got JSON body ({} chars):", json_str.len());
                    println!("{}", json_str);

                    // Parse JSON into Canvas
                    match serde_json_core::from_str::<Canvas>(json_str) {
                        Ok((canvas, _)) => {
                            println!(
                                "Successfully parsed canvas: {}x{}",
                                canvas.width, canvas.height
                            );
                            println!("Top-left pixel: {}", canvas.pixels[0][0].as_str());
                        }
                        Err(e) => println!("JSON parse error: {:?}", e),
                    }
                } else {
                    println!("No HTTP body separator found");
                }
            }
            Err(e) => println!("UTF-8 decode error: {:?}", e),
        }

        socket.disconnect();

        let deadline = time::Instant::now() + Duration::from_secs(5);
        while time::Instant::now() < deadline {
            socket.work();
        }
    }
}

fn timestamp() -> smoltcp::time::Instant {
    smoltcp::time::Instant::from_micros(
        esp_hal::time::Instant::now()
            .duration_since_epoch()
            .as_micros() as i64,
    )
}

pub fn create_interface(device: &mut esp_radio::wifi::WifiDevice) -> smoltcp::iface::Interface {
    smoltcp::iface::Interface::new(
        smoltcp::iface::Config::new(smoltcp::wire::HardwareAddress::Ethernet(
            smoltcp::wire::EthernetAddress::from_bytes(&device.mac_address()),
        )),
        device,
        timestamp(),
    )
}
