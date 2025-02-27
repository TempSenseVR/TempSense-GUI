// osc.rs
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use rosc::{OscPacket, OscType};

pub async fn osc_listener(addr: &str, sender: Sender<i8>) {
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

fn handle_packet(packet: OscPacket, sender: &Sender<i8>) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            if let Some(OscType::Float(value)) = msg.args.first() {
                println!("OSC Value: {}", value);
                let int_value = (*value * 100.0) as i8; // Convert f32 to i8 FOR TESTING. if we use ints, this needs to be updated. TODO:
                sender.send(int_value).unwrap(); 
      //          println!("Scaled {}", int_value);
            }
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}