// src/esp_comm.rs

use std::io::{self, Write, Read};
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;
use serialport::SerialPort;

// Commands that can be sent from the GUI thread to the ESP worker thread
#[derive(Debug)]
pub enum EspCommand {
    Connect(String, u32), // port_name, baud_rate
    Disconnect,
    SendCommand(String),
    StopThread,          // To gracefully shut down the thread
}

// Status messages that can be sent from the ESP worker thread to the GUI thread
#[derive(Debug)]
pub enum EspStatus {
    Connected,
    Disconnected(Option<String>), // Optional message for why (e.g., user action, error)
    Error(String),
    Message(String), // For data received from ESP or general info
}

pub fn esp_worker_thread(
    command_rx: Receiver<EspCommand>,
    status_tx: Sender<EspStatus>,
) {
    let mut serial_port: Option<Box<dyn SerialPort>> = None;
    let mut read_buffer: [u8; 1024] = [0; 1024];

    loop {
        match command_rx.try_recv() {
            Ok(cmd) => {
                match cmd {
                    EspCommand::Connect(port_name, baud_rate) => {
                        if serial_port.is_some() {
                            status_tx.send(EspStatus::Error("Already connected or connection attempt in progress.".to_string())).ok();
                            continue;
                        }
                        match serialport::new(&port_name, baud_rate)
                            .timeout(Duration::from_millis(1000))
                            .open()
                        {
                            Ok(port) => {
                                serial_port = Some(port);
                                status_tx.send(EspStatus::Connected).ok();
                            }
                            Err(e) => {
                                serial_port = None;
                                status_tx.send(EspStatus::Error(format!("Failed to connect to {}: {}", port_name, e))).ok();
                                // No break needed here as the thread didn't establish a working state to break from.
                            }
                        }
                    }
                    EspCommand::SendCommand(command_str) => {
                        if let Some(port) = serial_port.as_mut() {
                            let cmd_with_newline = format!("{}\n", command_str);
                            if let Err(e) = port.write_all(cmd_with_newline.as_bytes()) {
                                let error_msg = format!("Failed to send command: {}. Disconnecting.", e);
                                status_tx.send(EspStatus::Error(error_msg.clone())).ok();
                                serial_port.take(); 
                                status_tx.send(EspStatus::Disconnected(Some(error_msg))).ok();
                                break;
                            } else {
                                if let Err(e) = port.flush() {
                                     let error_msg = format!("Failed to flush serial port: {}. Disconnecting.", e);
                                     status_tx.send(EspStatus::Error(error_msg.clone())).ok();
                                     serial_port.take(); 
                                     status_tx.send(EspStatus::Disconnected(Some(error_msg))).ok();
                                     break;
                                }
                            }
                        } else {
                            status_tx.send(EspStatus::Error("Not connected to ESP. Cannot send command.".to_string())).ok();
                        }
                    }
                    EspCommand::Disconnect => {
                        if serial_port.take().is_some() { 
                            status_tx.send(EspStatus::Disconnected(Some("Disconnected by user.".to_string()))).ok();
                        } else {
                            status_tx.send(EspStatus::Message("Already disconnected.".to_string())).ok();
                        }
                        // If Disconnect command is from GUI, GUI expects worker to stop.
                        // The worker does this by no longer having a serial_port.
                        // To fully stop the thread, StopThread is preferred.
                        // However, after user disconnect, the main app will likely drop sender or send StopThread.
                        // For now, let's assume this is sufficient, or let StopThread handle full exit.
                        // If this command should also stop the thread, add 'break;'
                    }
                    EspCommand::StopThread => {
                        serial_port.take(); 
                        status_tx.send(EspStatus::Disconnected(Some("ESP worker thread stopped.".to_string()))).ok();
                        break; // Exit the loop, thread will terminate
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                if let Some(port) = serial_port.as_mut() {
                    match port.read(&mut read_buffer) {
                        Ok(bytes_read) if bytes_read > 0 => {
                            let message = String::from_utf8_lossy(&read_buffer[..bytes_read]).to_string();
                            status_tx.send(EspStatus::Message(message.trim().to_string())).ok();
                        }
                        Ok(_) => { /* 0 bytes read, no new data */ }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            // This is expected with a read timeout if no data is available
                        }
                        Err(e) => {
                            let error_msg = format!("Serial read error: {}. Disconnecting.", e);
                            status_tx.send(EspStatus::Error(error_msg.clone())).ok();
                            serial_port.take(); 
                            status_tx.send(EspStatus::Disconnected(Some(error_msg))).ok();
                            break;
                        }
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                serial_port.take(); 
                break; 
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
}
