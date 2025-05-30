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
    let mut read_buffer: [u8; 1024] = [0; 1024]; // Buffer for reading serial data

    loop {
        match command_rx.try_recv() {
            Ok(cmd) => {
                // Process command
                match cmd {
                    EspCommand::Connect(port_name, baud_rate) => {
                        if serial_port.is_some() {
                            status_tx.send(EspStatus::Error("Already connected or connection attempt in progress.".to_string())).ok();
                            continue;
                        }
                        match serialport::new(&port_name, baud_rate)
                            .timeout(Duration::from_millis(1000)) // Connection timeout
                            .open()
                        {
                            Ok(port) => {
                                serial_port = Some(port);
                                status_tx.send(EspStatus::Connected).ok();
                            }
                            Err(e) => {
                                serial_port = None;
                                status_tx.send(EspStatus::Error(format!("Failed to connect to {}: {}", port_name, e))).ok();
                            }
                        }
                    }
                    EspCommand::SendCommand(command_str) => {
                        if let Some(port) = serial_port.as_mut() {
                            let cmd_with_newline = format!("{}\n", command_str); // ESPs often expect a newline
                            if let Err(e) = port.write_all(cmd_with_newline.as_bytes()) {
                                status_tx.send(EspStatus::Error(format!("Failed to send command: {}", e))).ok();
                            } else {
                                if let Err(e) = port.flush() {
                                     status_tx.send(EspStatus::Error(format!("Failed to flush serial port: {}", e))).ok();
                                } else {
                                    // Optionally, confirm command was sent, or wait for an ACK if your ESP sends one.
                                    // For now, just assume sent if no error.
                                    // status_tx.send(EspStatus::Message(format!("Sent: {}", command_str))).ok();
                                }
                            }
                        } else {
                            status_tx.send(EspStatus::Error("Not connected to ESP. Cannot send command.".to_string())).ok();
                        }
                    }
                    EspCommand::Disconnect => {
                        if serial_port.take().is_some() { // take() consumes the value, effectively dropping/closing the port
                            status_tx.send(EspStatus::Disconnected(Some("Disconnected by user.".to_string()))).ok();
                        } else {
                            status_tx.send(EspStatus::Message("Already disconnected.".to_string())).ok();
                        }
                    }
                    EspCommand::StopThread => {
                        serial_port.take(); // Ensure port is closed
                        status_tx.send(EspStatus::Disconnected(Some("ESP worker thread stopped.".to_string()))).ok();
                        break; // Exit the loop, thread will terminate
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                // No command from GUI, try to read from serial if connected
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
                            // Handle other read errors (e.g., device disconnected)
                            status_tx.send(EspStatus::Error(format!("Serial read error: {}. Disconnecting.", e))).ok();
                            serial_port.take(); // Close the port
                            status_tx.send(EspStatus::Disconnected(Some(format!("Disconnected due to read error: {}", e)))).ok();

                        }
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                // GUI thread likely closed, or sender was dropped.
                serial_port.take(); // Ensure port is closed
                break; // Exit the loop
            }
        }
        // Small sleep to prevent the loop from spinning too fast when idle and using try_recv
        // Adjust based on responsiveness needs and serial read behavior
        thread::sleep(Duration::from_millis(20));
    }
}