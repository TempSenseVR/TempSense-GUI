// osc.rs
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use rosc::{OscPacket, OscType};

pub async fn osc_listener(addr: &str, sender: Sender<(i8, i8)>) {
    let usage = format!("Usage: {} IP:PORT", addr);

    let socket_addr = match SocketAddrV4::from_str(addr) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("{}", usage);
            std::process::exit(1);
        }
    };

    let sock = UdpSocket::bind(socket_addr).unwrap();
    println!("Listening on {}", socket_addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, sender_addr)) => {
                println!("Received packet with size {} from: {}", size, sender_addr);
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    handle_packet(packet, &sender);
                } else {
                    eprintln!("Failed to decode OSC packet");
                }
            }
            Err(e) => {
                eprintln!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}

fn handle_packet(packet: OscPacket, sender: &Sender<(i8, i8)>) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            if let Some(OscType::Float(value)) = msg.args.first() {
                println!("OSC Value: {}", value);
                let int_value = (*value * 100.0) as i8; // Convert f32 to i8 FOR TESTING. if we use ints, this needs to be updated. TODO:
                let id: i8; // Peltier id. We are only using Pelt1 and Pelt2 for now. - David
                let addr_str = msg.addr.as_str();
                match addr_str {
                    "/Pelt1" => id = 0,
                    "/Pelt2" => id = 1,
                    "/Pelt3" => id = 2,
                    "/Pelt4" => id = 3,
                    "/Pelt5" => id = 4,
                    "/Pelt6" => id = 5,
                    "/Pelt7" => id = 6,
                    "/Pelt8" => id = 7,
                    _      => {
                        println!("[osc.rs] WARNING: Address '{}' did not match specific /PeltX. Defaulting id to 0.", addr_str);
                        id = 0;
                    }
                }

                let address_msg_tuple: (i8, i8) = (id, int_value);
                sender.send(address_msg_tuple).unwrap(); 
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}