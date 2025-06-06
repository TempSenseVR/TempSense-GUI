// src/app.rs

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::esp_comm::{EspCommand, EspStatus, esp_worker_thread};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Page {
    Home,
    OscSettings,
    EspConnection,
    AppSettings
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    pub osc_ip: String,
    #[serde(skip)]
    pub value: f32,
    pub value_max: i8,
    pub value_min: i8,
    pub osc_port: String,
    #[serde(skip)]
    pub is_running: bool,
    
    // ESP 1 (Peltier 1)
    pub esp_port_1: String, 
    #[serde(skip)]
    pub pelt_temp_1: i8,
    #[serde(skip)]
    pub pelt_temp_1_old: i8,
    #[serde(skip)]
    pub esp_command_sender_1: Option<Sender<EspCommand>>,
    #[serde(skip)]
    pub esp_status_receiver_1: Option<Receiver<EspStatus>>,
    #[serde(skip)]
    pub esp_thread_handle_1: Option<JoinHandle<()>>,
    #[serde(skip)]
    pub esp_connected_1: bool,
    #[serde(skip)]
    pub esp_status_message_1: String,
    pub esp_baud_rate_1: u32,

    // ESP 2 (Peltier 2)
    pub esp_port_2: String, 
    #[serde(skip)]
    pub pelt_temp_2: i8,
    #[serde(skip)]
    pub pelt_temp_2_old: i8,
    #[serde(skip)]
    pub esp_command_sender_2: Option<Sender<EspCommand>>,
    #[serde(skip)]
    pub esp_status_receiver_2: Option<Receiver<EspStatus>>,
    #[serde(skip)]
    pub esp_thread_handle_2: Option<JoinHandle<()>>,
    #[serde(skip)]
    pub esp_connected_2: bool,
    #[serde(skip)]
    pub esp_status_message_2: String,
    pub esp_baud_rate_2: u32,

    #[serde(skip)]
    pub osc_receiver: Receiver<(i8, i8)>,
    #[serde(skip)]
    pub last_update_time: std::time::Instant,
    #[serde(skip)]
    pub current_page: Page,
    #[serde(skip)]
    pub esp_log: Vec<String>, // Shared log for messages from ESPs and app

    #[serde(skip)]
    pub manual_pelt_1_temp_str: String,
    #[serde(skip)]
    pub manual_pelt_2_temp_str: String,

    #[serde(skip)]
    pub skin_temp_1: Option<f32>, 
    #[serde(skip)] 
    pub skin_temp_2: Option<f32>,

    #[serde(skip)]
    pub manual_pelt_1: bool,
    #[serde(skip)]
    pub manual_pelt_2: bool
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (_, osc_receiver) = mpsc::channel();
        Self {
            osc_ip: "127.0.0.1".to_owned(),
            value: 2.7,
            value_max: 40,
            value_min: -10,
            osc_port: "9000".to_owned(),
            is_running: false,
            
            // ESP 1 (Peltier 1)
            esp_port_1: if cfg!(windows) { "COM3".to_string() } else { "/dev/ttyUSB0".to_string() },
            pelt_temp_1: 0,
            pelt_temp_1_old: -127,
            esp_command_sender_1: None,
            esp_status_receiver_1: None,
            esp_thread_handle_1: None,
            esp_connected_1: false,
            esp_status_message_1: "ESP1: Not connected.".to_string(),
            esp_baud_rate_1: 115200,

            // ESP 2 (Peltier 2)
            esp_port_2: if cfg!(windows) { "COM4".to_string() } else { "/dev/ttyUSB1".to_string() },
            pelt_temp_2: 0,
            pelt_temp_2_old: -127,
            esp_command_sender_2: None,
            esp_status_receiver_2: None,
            esp_thread_handle_2: None,
            esp_connected_2: false,
            esp_status_message_2: "ESP2: Not connected.".to_string(),
            esp_baud_rate_2: 115200,

            last_update_time: std::time::Instant::now(),
            osc_receiver,
            current_page: Page::Home,
            esp_log: Vec::new(),

            manual_pelt_1_temp_str: "0".to_string(),
            manual_pelt_2_temp_str: "0".to_string(),
            skin_temp_1: None,
            skin_temp_2: None,
            manual_pelt_1: false,
            manual_pelt_2: false
        }
    }
}

impl TemplateApp {
    pub fn update_pelt_temp(&mut self, _id: i8, temp: i8) {
        match _id {
            0 => {
                if self.manual_pelt_1 != true {
                    self.pelt_temp_1 = temp;
                    println!("OSC temp update for Peltier 0: {:?}", temp); // Added print here
                }
            },
            1 => {
                if self.manual_pelt_2 != true {
                    self.pelt_temp_2 = temp;
                    println!("OSC temp update for Peltier 1: {:?}", temp); // Added print here
                }

            },
            _ => {
                // This is for invalid _id
                println!("OSC temp received with INVALID _id ({}): {:?}. Defaulting to pelt_temp_1", _id, temp);
                self.add_esp_log_message("APP", format!("Invalid peltier _id: {}. Defaulting to pelt_temp_1", _id));
                self.pelt_temp_1 = temp;
            },
        }
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }

    // Render the Home page content
fn render_home_page(&mut self, ui: &mut egui::Ui) {
        // Peltier 1
        ui.horizontal(|ui| {
            ui.label("Pelt 1:");
            ui.visuals_mut().override_text_color = Some(if self.is_running && self.esp_connected_1 { egui::Color32::GREEN } else { egui::Color32::LIGHT_RED });
            ui.label(if self.is_running && self.esp_connected_1 { "ON" } else { "OFF" });
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY); 
            ui.label("Temp:");
            let actual_temp_str_1 = self.skin_temp_1.map_or_else(
                || "--.-°C".to_string(), 
                |temp| format!("{:.1}°C", temp)
            );
            ui.label(actual_temp_str_1);
            ui.label("➡ "); 
            ui.label(format!("{}°C", self.pelt_temp_1));

            if self.esp_connected_1 && self.pelt_temp_1 != self.pelt_temp_1_old {
                if let Some(sender) = &self.esp_command_sender_1 {
                    let command_to_send = format!("setTemp {}", self.pelt_temp_1);
                    if let Err(e) = sender.send(EspCommand::SendCommand(command_to_send.clone())) {
                        self.esp_status_message_1 = format!("ESP1: Error sending command: {:?}", e);
                        self.add_esp_log_message("ESP1", format!("Failed to send '{}': {:?}", command_to_send, e));
                    } else {
                        self.add_esp_log_message("ESP1", format!("Sent command: {}", command_to_send));
                    }
                }
            } else if self.pelt_temp_1 != self.pelt_temp_1_old && !self.esp_connected_1 { // only log if temp changed
                self.esp_status_message_1 = "ESP1: Not connected.".to_string();
                self.add_esp_log_message("ESP1", "Attempted to send command while ESP1 not connected.".to_string());
            }
            self.pelt_temp_1_old = self.pelt_temp_1;
        });
        ui.visuals_mut().override_text_color = None;
            
        // Peltier 2
        ui.horizontal(|ui| {
            ui.label("Pelt 2:");
            ui.visuals_mut().override_text_color = Some(if self.is_running && self.esp_connected_2 { egui::Color32::GREEN } else { egui::Color32::LIGHT_RED });
            ui.label(if self.is_running && self.esp_connected_2 { "ON" } else { "OFF" });
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
            ui.label("Temp:");
            let actual_temp_str_2 = self.skin_temp_2.map_or_else(
                || "--.-°C".to_string(),
                |temp| format!("{:.1}°C", temp)
            );
            ui.label(actual_temp_str_2);
            ui.label("➡ ");
            ui.label(format!("{}°C", self.pelt_temp_2));

            if self.esp_connected_2 && self.pelt_temp_2 != self.pelt_temp_2_old {
                if let Some(sender) = &self.esp_command_sender_2 {
                    let command_to_send = format!("setTemp {}", self.pelt_temp_2);
                    if let Err(e) = sender.send(EspCommand::SendCommand(command_to_send.clone())) {
                        self.esp_status_message_2 = format!("ESP2: Error sending command: {:?}", e);
                        self.add_esp_log_message("ESP2", format!("Failed to send '{}': {:?}", command_to_send, e));
                    } else {
                        self.add_esp_log_message("ESP2", format!("Sent command: {}", command_to_send));
                    }
                }
            } else if self.pelt_temp_2 != self.pelt_temp_2_old && !self.esp_connected_2 { // only log if temp changed
                self.esp_status_message_2 = "ESP2: Not connected.".to_string();
                self.add_esp_log_message("ESP2", "Attempted to send command while ESP2 not connected.".to_string());
            }
            self.pelt_temp_2_old = self.pelt_temp_2;
        });
        ui.visuals_mut().override_text_color = None;

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("START ▶").clicked() {
                self.is_running = true;
                let mut s1_msg_set = false;
                let mut s2_msg_set = false;

                if self.esp_connected_1 {
                    if let Some(sender) = &self.esp_command_sender_1 {
                        if let Err(e) = sender.send(EspCommand::SendCommand("tempActive 1".to_string())) {
                            self.esp_status_message_1 = format!("ESP1: Error sending START: {}", e);
                            self.add_esp_log_message("ESP1", format!("Error sending START: {}", e));
                            s1_msg_set = true;
                        } else {
                             self.esp_status_message_1 = "ESP1: START command sent.".to_string();
                             self.add_esp_log_message("ESP1", "START command sent.".to_string());
                             s1_msg_set = true;
                        }
                    }
                } else {
                    self.esp_status_message_1 = "ESP1: Cannot START, not connected.".to_string();
                    self.add_esp_log_message("ESP1", "Attempted START while ESP1 not connected.".to_string());
                    s1_msg_set = true;
                }

                if self.esp_connected_2 {
                    if let Some(sender) = &self.esp_command_sender_2 {
                        if let Err(e) = sender.send(EspCommand::SendCommand("tempActive 1".to_string())) {
                            self.esp_status_message_2 = format!("ESP2: Error sending START: {}", e);
                            self.add_esp_log_message("ESP2", format!("Error sending START: {}", e));
                            s2_msg_set = true;
                        } else {
                             self.esp_status_message_2 = "ESP2: START command sent.".to_string();
                             self.add_esp_log_message("ESP2", "START command sent.".to_string());
                             s2_msg_set = true;
                        }
                    }
                } else {
                    self.esp_status_message_2 = "ESP2: Cannot START, not connected.".to_string();
                    self.add_esp_log_message("ESP2", "Attempted START while ESP2 not connected.".to_string());
                    s2_msg_set = true;
                }
                 if !s1_msg_set { self.esp_status_message_1 = "ESP1: Status unchanged.".to_string(); }
                 if !s2_msg_set { self.esp_status_message_2 = "ESP2: Status unchanged.".to_string(); }
            }
            if ui.button("STOP ALL ■").clicked() {
                self.is_running = false;
                let mut s1_msg_set = false;
                let mut s2_msg_set = false;

                if self.esp_connected_1 {
                    if let Some(sender) = &self.esp_command_sender_1 {
                        if let Err(e) = sender.send(EspCommand::SendCommand("tempActive 0".to_string())) {
                            self.esp_status_message_1 = format!("ESP1: Error sending STOP: {}", e);
                            self.add_esp_log_message("ESP1", format!("Error sending STOP: {}",e));
                            s1_msg_set = true;
                        } else {
                            self.esp_status_message_1 = "ESP1: STOP command sent.".to_string();
                            self.add_esp_log_message("ESP1", "STOP command sent.".to_string());
                            s1_msg_set = true;
                        }
                    }
                } else {
                    self.esp_status_message_1 = "ESP1: Cannot STOP, not connected.".to_string();
                    self.add_esp_log_message("ESP1", "Attempted STOP while ESP1 not connected.".to_string());
                    s1_msg_set = true;
                }

                if self.esp_connected_2 {
                    if let Some(sender) = &self.esp_command_sender_2 {
                        if let Err(e) = sender.send(EspCommand::SendCommand("tempActive 0".to_string())) {
                            self.esp_status_message_2 = format!("ESP2: Error sending STOP: {}", e);
                            self.add_esp_log_message("ESP2", format!("Error sending STOP: {}",e));
                            s2_msg_set = true;
                        } else {
                            self.esp_status_message_2 = "ESP2: STOP command sent.".to_string();
                            self.add_esp_log_message("ESP2", "STOP command sent.".to_string());
                            s2_msg_set = true;
                        }
                    }
                } else {
                    self.esp_status_message_2 = "ESP2: Cannot STOP, not connected.".to_string();
                    self.add_esp_log_message("ESP2", "Attempted STOP while ESP2 not connected.".to_string());
                    s2_msg_set = true;
                }
                if !s1_msg_set { self.esp_status_message_1 = "ESP1: Status unchanged.".to_string(); }
                if !s2_msg_set { self.esp_status_message_2 = "ESP2: Status unchanged.".to_string(); }
            }
        });

        ui.horizontal(|ui| {
            ui.label("System Status:");
            if self.is_running {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("RUNNING");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("STOPPED");
            }
        });
         ui.visuals_mut().override_text_color = None; 

        ui.horizontal(|ui| {
            ui.label("OSC: ");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN); 
            ui.label("READY"); 
        });
        ui.visuals_mut().override_text_color = None; 

        ui.horizontal(|ui| {
            ui.label("ESP 1: ");
            if self.esp_connected_1 {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("CONNECTED");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("DISCONNECTED");
            }
        });
        ui.visuals_mut().override_text_color = None; 
        ui.label(&self.esp_status_message_1);

        ui.horizontal(|ui| {
            ui.label("ESP 2: ");
            if self.esp_connected_2 {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("CONNECTED");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("DISCONNECTED");
            }
        });
        ui.visuals_mut().override_text_color = None; 
        ui.label(&self.esp_status_message_2);

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Manual Pelt 1 temp: ");
            ui.add(egui::TextEdit::singleline(&mut self.manual_pelt_1_temp_str).desired_width(50.0));

            if ui.button("Set Temp").clicked() {
                if let Ok(temp_val) = self.manual_pelt_1_temp_str.parse::<i8>() {
                    if self.pelt_temp_1 != temp_val { // Only if value actually changes
                        self.pelt_temp_1 = temp_val;
                        self.add_esp_log_message("APP", format!("Manual override: Peltier 1 target directly set to {}°C", temp_val));
                        ui.ctx().request_repaint(); // Ensure repaint for immediate feedback and re-evaluation
                    }
                } else {
                    self.add_esp_log_message("APP", format!("Invalid temperature input for Peltier 1: '{}'", self.manual_pelt_1_temp_str));
                }
            }
            ui.checkbox(&mut self.manual_pelt_1, "Override OSC");
        });

        ui.horizontal(|ui| {
            ui.label("Manual Pelt 2 temp: ");
            ui.add(egui::TextEdit::singleline(&mut self.manual_pelt_2_temp_str).desired_width(50.0));

            if ui.button("Set Temp").clicked() {
                if let Ok(temp_val) = self.manual_pelt_2_temp_str.parse::<i8>() {
                    if self.pelt_temp_2 != temp_val { // Only if value actually changes
                        self.pelt_temp_2 = temp_val;
                        self.add_esp_log_message("APP", format!("Manual override: Peltier 2 target directly set to {}°C", temp_val));
                        ui.ctx().request_repaint(); // Ensure repaint for immediate feedback and re-evaluation
                    }
                } else {
                    self.add_esp_log_message("APP", format!("Invalid temperature input for Peltier 2: '{}'", self.manual_pelt_2_temp_str));
                }
            }
            ui.checkbox(&mut self.manual_pelt_2, "Override OSC");
        });
    }
    
     fn render_osc_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSC Settings");
        
        ui.horizontal(|ui| {
            ui.label("OSC IP Address:");
            ui.add(egui::TextEdit::singleline(&mut self.osc_ip).desired_width(150.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("OSC Port:");
            ui.add(egui::TextEdit::singleline(&mut self.osc_port).desired_width(100.0));
        });
        
        ui.add_space(20.0);
        
        if ui.button("Apply OSC Settings").clicked() {
             // For now, just log it. Actual implementation of applying OSC settings would go here.
             self.add_esp_log_message("APP", "OSC Settings Applied (Placeholder).".to_string());
        }
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("OSC Status:");
            // TODO: Implement actual OSC connection status logic
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN); // Placeholder
            ui.label("READY"); // Placeholder
        });
        ui.visuals_mut().override_text_color = None; 
    }


    fn render_esp_connection_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("ESP Connections");
        ui.separator();

        // --- ESP 1 (Peltier 1) ---
        ui.heading("ESP 1 (Peltier 1)");
        let port_text_edit_1 = egui::TextEdit::singleline(&mut self.esp_port_1).desired_width(150.0);
        ui.horizontal(|ui| {
            ui.label("ESP 1 Serial Port:");
            ui.add_enabled(self.esp_thread_handle_1.is_none(), port_text_edit_1); 
        });

        let mut baud_str_edit_1 = self.esp_baud_rate_1.to_string();
         ui.horizontal(|ui| {
            ui.label("ESP 1 Baud Rate:");
            let response = ui.add_enabled(
                self.esp_thread_handle_1.is_none(),
                egui::TextEdit::singleline(&mut baud_str_edit_1).desired_width(100.0)
            );
            if response.changed() {
                if let Ok(new_baud) = baud_str_edit_1.parse::<u32>() {
                    self.esp_baud_rate_1 = new_baud;
                }
            }
        });

        if self.esp_thread_handle_1.is_none() { 
            if ui.button("Connect to ESP 1").clicked() {
                let (command_s, command_r) = mpsc::channel();
                let (status_s, status_r) = mpsc::channel();
                self.esp_command_sender_1 = Some(command_s.clone());
                self.esp_status_receiver_1 = Some(status_r);
                let port_name_clone = self.esp_port_1.clone();
                let baud_rate_clone = self.esp_baud_rate_1;
                
                self.esp_thread_handle_1 = Some(thread::spawn(move || {
                    esp_worker_thread(command_r, status_s); // This worker thread now implicitly handles ESP1
                }));
                
                let connect_msg = format!("Attempting to connect to ESP1 @ {} ({} baud)...", self.esp_port_1, self.esp_baud_rate_1);
                if let Err(e) = command_s.send(EspCommand::Connect(port_name_clone, baud_rate_clone)) {
                     self.esp_status_message_1 = format!("ESP1: Failed to send connect cmd: {}",e);
                     self.add_esp_log_message("ESP1", format!("Failed to send connect cmd: {}",e));
                     self.esp_command_sender_1 = None;
                     self.esp_status_receiver_1 = None;
                     self.esp_thread_handle_1.take();
                } else {
                    self.esp_status_message_1 = connect_msg.clone();
                    self.add_esp_log_message("ESP1", connect_msg);
                }
            }
        } else {
            if ui.button("Disconnect from ESP 1").clicked() {
                if let Some(sender) = &self.esp_command_sender_1 {
                    if let Err(e) = sender.send(EspCommand::Disconnect) {
                         self.esp_status_message_1 = format!("ESP1: Failed to send disconnect cmd: {}",e);
                         self.add_esp_log_message("ESP1", format!("Failed to send disconnect cmd: {}",e));
                    } else {
                        self.esp_status_message_1 = "ESP1: Disconnect command sent.".to_string();
                        self.add_esp_log_message("ESP1", "Disconnect command sent.".to_string());
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            ui.label("ESP 1 Status:");
            if self.esp_connected_1 {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("CONNECTED");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("DISCONNECTED");
            }
        });

        ui.visuals_mut().override_text_color = None; 
        ui.label(&self.esp_status_message_1);
        #[cfg(debug_assertions)] 
        if self.esp_connected_1 {
            if ui.button("Send 'PING' to ESP 1").clicked() {
                 if let Some(sender) = &self.esp_command_sender_1 {
                    if let Err(e) = sender.send(EspCommand::SendCommand("PING".to_string())) {
                        self.add_esp_log_message("ESP1", format!("Error sending PING: {}", e));
                    } else {
                        self.add_esp_log_message("ESP1", "Sent PING to ESP1.".to_string());
                    }
                 }
            }
        }
        
        ui.separator();

        // --- ESP 2 (Peltier 2) ---
        ui.heading("ESP 2 (Peltier 2)");
        let port_text_edit_2 = egui::TextEdit::singleline(&mut self.esp_port_2).desired_width(150.0);
        ui.horizontal(|ui| {
            ui.label("ESP 2 Serial Port:");
            ui.add_enabled(self.esp_thread_handle_2.is_none(), port_text_edit_2); 
        });

        let mut baud_str_edit_2 = self.esp_baud_rate_2.to_string();
         ui.horizontal(|ui| {
            ui.label("ESP 2 Baud Rate:");
            let response = ui.add_enabled(
                self.esp_thread_handle_2.is_none(),
                egui::TextEdit::singleline(&mut baud_str_edit_2).desired_width(100.0)
            );
            if response.changed() {
                if let Ok(new_baud) = baud_str_edit_2.parse::<u32>() {
                    self.esp_baud_rate_2 = new_baud;
                }
            }
        });

        if self.esp_thread_handle_2.is_none() { 
            if ui.button("Connect to ESP 2").clicked() {
                let (command_s, command_r) = mpsc::channel();
                let (status_s, status_r) = mpsc::channel();
                self.esp_command_sender_2 = Some(command_s.clone());
                self.esp_status_receiver_2 = Some(status_r);
                let port_name_clone = self.esp_port_2.clone();
                let baud_rate_clone = self.esp_baud_rate_2;
                
                self.esp_thread_handle_2 = Some(thread::spawn(move || {
                    esp_worker_thread(command_r, status_s); // This worker thread now implicitly handles ESP2
                }));
                
                let connect_msg = format!("Attempting to connect to ESP2 @ {} ({} baud)...", self.esp_port_2, self.esp_baud_rate_2);
                if let Err(e) = command_s.send(EspCommand::Connect(port_name_clone, baud_rate_clone)) {
                     self.esp_status_message_2 = format!("ESP2: Failed to send connect cmd: {}",e);
                     self.add_esp_log_message("ESP2", format!("Failed to send connect cmd: {}",e));
                     self.esp_command_sender_2 = None;
                     self.esp_status_receiver_2 = None;
                     self.esp_thread_handle_2.take();
                } else {
                    self.esp_status_message_2 = connect_msg.clone();
                    self.add_esp_log_message("ESP2", connect_msg);
                }
            }
        } else {
            if ui.button("Disconnect from ESP 2").clicked() {
                if let Some(sender) = &self.esp_command_sender_2 {
                    if let Err(e) = sender.send(EspCommand::Disconnect) {
                         self.esp_status_message_2 = format!("ESP2: Failed to send disconnect cmd: {}",e);
                         self.add_esp_log_message("ESP2", format!("Failed to send disconnect cmd: {}",e));
                    } else {
                        self.esp_status_message_2 = "ESP2: Disconnect command sent.".to_string();
                        self.add_esp_log_message("ESP2", "Disconnect command sent.".to_string());
                    }
                }
            }
        }
        ui.horizontal(|ui| {
            ui.label("ESP 2 Status:");
            if self.esp_connected_2 {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("CONNECTED");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("DISCONNECTED");
            }
        });
        ui.visuals_mut().override_text_color = None; 
        ui.label(&self.esp_status_message_2);

        #[cfg(debug_assertions)] 
        if self.esp_connected_2 {
            if ui.button("Send 'PING' to ESP 2").clicked() {
                 if let Some(sender) = &self.esp_command_sender_2 {
                    if let Err(e) = sender.send(EspCommand::SendCommand("PING".to_string())) {
                        self.add_esp_log_message("ESP2", format!("Error sending PING: {}", e));
                    } else {
                        self.add_esp_log_message("ESP2", "Sent PING to ESP2.".to_string());
                    }
                 }
            }
        }
        
        ui.add_space(10.0);
        ui.separator();
        ui.label("ESP Log/Messages (Shared):");
        egui::ScrollArea::vertical().max_height(150.0).stick_to_bottom(true).show(ui, |ui| {
            for msg in self.esp_log.iter() { 
                ui.label(msg);
            }
        });
    }
    
    fn render_app_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("App Settings");
        ui.separator();
        
        egui::widgets::global_theme_preference_buttons(ui);
    }

    // Added esp_identifier to distinguish log messages
    fn add_esp_log_message(&mut self, esp_identifier: &str, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        self.esp_log.push(format!("[{}] [{}] {}", timestamp, esp_identifier, message));
        if self.esp_log.len() > 200 { // Keep the log size manageable
            self.esp_log.remove(0);
        }
    }

    fn parse_esp_message_and_update_state(&mut self, esp_id_str: &str, msg: &str) {
        // Example msg: "Skin_Temp_Smoothed:12.12,Exterior_Temp:18.73,Target_Temp:10.0,Heat_PID_output:0.0,Cool_PID_output:64.4,Ambient:22.0"
        for part in msg.split(',') {
            let mut kv_iterator = part.splitn(2, ':');
            if let (Some(key_raw), Some(value_raw)) = (kv_iterator.next(), kv_iterator.next()) {
                let key = key_raw.trim();
                let value_str = value_raw.trim();

                if key == "Skin_Temp_Smoothed" {
                    if let Ok(temp_f32) = value_str.parse::<f32>() {
                        if esp_id_str == "ESP1" {
                            self.skin_temp_1 = Some(temp_f32);
                        } else if esp_id_str == "ESP2" {
                            self.skin_temp_2 = Some(temp_f32);
                        }
                    } else {
                        self.add_esp_log_message(esp_id_str, format!("Failed to parse Skin_Temp_Smoothed value: '{}'", value_str));
                    }
                }
                // Future: Add parsing for other values like "Exterior_Temp", "Target_Temp" here
            }
        }
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incoming OSC messages
// Process ALL available OSC messages this frame
        while let Ok(osc_id_and_message) = self.osc_receiver.try_recv() {
            self.update_pelt_temp(osc_id_and_message.0, osc_id_and_message.1);
        }
        
        let mut processed_any_message_this_frame = false;

        // Process incoming ESP 1 status messages
        let receiver1_temp_opt = self.esp_status_receiver_1.take();
        let mut clear_receiver1_permanently = false; 
        if let Some(ref rx1) = receiver1_temp_opt { 
            while let Ok(status) = rx1.try_recv() {
                processed_any_message_this_frame = true;
                match status {
                    EspStatus::Connected => {
                        self.esp_connected_1 = true;
                        self.esp_status_message_1 = "ESP1 Connected.".to_string();
                        self.add_esp_log_message("ESP1", "Connected.".to_string());
                    }
                    EspStatus::Disconnected(reason) => {
                        self.esp_connected_1 = false;
                        let msg = reason.unwrap_or_else(|| "Disconnected by worker.".to_string());
                        self.esp_status_message_1 = format!("ESP1: {}", msg);
                        self.add_esp_log_message("ESP1", msg);

                        if let Some(handle) = self.esp_thread_handle_1.take() {
                             let _ = handle.join().map_err(|e| self.add_esp_log_message("ESP1", format!("Thread panicked or error on join: {:?}", e)));
                        }
                        self.esp_command_sender_1 = None;
                        clear_receiver1_permanently = true; 
                    }
                    EspStatus::Error(err_msg) => {
                        let full_err_msg = format!("Error: {}", err_msg);
                        self.esp_status_message_1 = format!("ESP1: {}",full_err_msg);
                        self.add_esp_log_message("ESP1", full_err_msg);
                    }
                    EspStatus::Message(msg) => {
                        self.add_esp_log_message("ESP1", format!("MSG: {}", msg));
                        self.parse_esp_message_and_update_state("ESP1", &msg);
                    }
                }
            }
        }

        if !clear_receiver1_permanently && receiver1_temp_opt.is_some() {
            self.esp_status_receiver_1 = receiver1_temp_opt;
        } 
        else if clear_receiver1_permanently {
             if self.esp_thread_handle_1.is_some() {
                if let Some(handle) = self.esp_thread_handle_1.take() {
                    self.add_esp_log_message("ESP1", "Ensuring thread is joined after disconnect (update).".to_string());
                    let _ = handle.join().map_err(|e| self.add_esp_log_message("ESP1", format!("Thread panicked/error on join (update): {:?}", e)));
                }
             }
             if self.esp_command_sender_1.is_some() && self.esp_thread_handle_1.is_none() {
                self.esp_command_sender_1 = None;
                self.add_esp_log_message("ESP1", "Cleared command sender as thread handle is gone.".to_string());
             }
        }

        // Process incoming ESP 2 status messages
        let receiver2_temp_opt = self.esp_status_receiver_2.take();
        let mut clear_receiver2_permanently = false;
        if let Some(ref rx2) = receiver2_temp_opt {
            while let Ok(status) = rx2.try_recv() {
                processed_any_message_this_frame = true;
                match status {
                    EspStatus::Connected => {
                        self.esp_connected_2 = true;
                        self.esp_status_message_2 = "ESP2 Connected.".to_string();
                        self.add_esp_log_message("ESP2", "Connected.".to_string());
                    }
                    EspStatus::Disconnected(reason) => {
                        self.esp_connected_2 = false;
                        let msg = reason.unwrap_or_else(|| "Disconnected by worker.".to_string());
                        self.esp_status_message_2 = format!("ESP2: {}", msg);
                        self.add_esp_log_message("ESP2", msg);

                        if let Some(handle) = self.esp_thread_handle_2.take() {
                             let _ = handle.join().map_err(|e| self.add_esp_log_message("ESP2", format!("Thread panicked or error on join: {:?}", e)));
                        }
                        self.esp_command_sender_2 = None;
                        clear_receiver2_permanently = true;
                    }
                    EspStatus::Error(err_msg) => {
                        let full_err_msg = format!("Error: {}", err_msg);
                        self.esp_status_message_2 = format!("ESP2: {}",full_err_msg);
                        self.add_esp_log_message("ESP2", full_err_msg);
                    }
                    EspStatus::Message(msg) => {
                        self.parse_esp_message_and_update_state("ESP2", &msg);
                        self.add_esp_log_message("ESP2", format!("MSG: {}", msg));
                    }
                }
            }
        }

        if !clear_receiver2_permanently && receiver2_temp_opt.is_some() {
            self.esp_status_receiver_2 = receiver2_temp_opt;
        } 
        else if clear_receiver2_permanently {
             if self.esp_thread_handle_2.is_some() {
                if let Some(handle) = self.esp_thread_handle_2.take() {
                    self.add_esp_log_message("ESP2", "Ensuring thread is joined after disconnect (update).".to_string());
                    let _ = handle.join().map_err(|e| self.add_esp_log_message("ESP2", format!("Thread panicked/error on join (update): {:?}", e)));
                }
             }
             if self.esp_command_sender_2.is_some() && self.esp_thread_handle_2.is_none() {
                self.esp_command_sender_2 = None;
                self.add_esp_log_message("ESP2", "Cleared command sender as thread handle is gone.".to_string());
             }
        }

        if self.osc_receiver.try_recv().is_ok() || processed_any_message_this_frame {
            ctx.request_repaint();
        } 
        else {
            ctx.request_repaint_after_for(Duration::from_millis(75), ctx.viewport_id());
        }


        egui::TopBottomPanel::top("page_navigation").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let button_height = 32.0;
                let button_width = 100.0;
 
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.spacing_mut().button_padding = egui::vec2(0.0, 8.0);
                    
                    if ui.add_sized([button_width, button_height], egui::SelectableLabel::new(self.current_page == Page::Home, "Home")).clicked() {
                        self.current_page = Page::Home;
                    }
                    if ui.add_sized([button_width, button_height], egui::SelectableLabel::new(self.current_page == Page::OscSettings, "OSC Settings")).clicked() {
                        self.current_page = Page::OscSettings;
                    }
                    if ui.add_sized([button_width, button_height], egui::SelectableLabel::new(self.current_page == Page::EspConnection, "ESP Connection")).clicked() {
                        self.current_page = Page::EspConnection;
                    }
                    if ui.add_sized([button_width, button_height], egui::SelectableLabel::new(self.current_page == Page::AppSettings, "App Settings")).clicked() {
                        self.current_page = Page::AppSettings;
                    }
                });
                ui.add_space(8.0);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_page {
                Page::Home => self.render_home_page(ui),
                Page::OscSettings => self.render_osc_settings_page(ui),
                Page::EspConnection => self.render_esp_connection_page(ui),
                Page::AppSettings => self.render_app_settings_page(ui),
            }
            ui.separator();
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.add_esp_log_message("APP", "Application exiting. Stopping ESP workers.".to_string());
        
        // ESP 1 shutdown
        if let Some(sender) = self.esp_command_sender_1.take() {
            // Attempt to send StopThread, ignore error if channel already closed (e.g., worker already exited)
            let _ = sender.send(EspCommand::StopThread); 
        }
        if let Some(handle) = self.esp_thread_handle_1.take() {
           
            if let Err(e) = handle.join().map_err(|e_join| format!("ESP1 thread panicked or error on join: {:?}", e_join)) {
                self.add_esp_log_message("ESP1", e); 
            }
        }

        // ESP 2 shutdown
        if let Some(sender) = self.esp_command_sender_2.take() {
            let _ = sender.send(EspCommand::StopThread);
        }
        if let Some(handle) = self.esp_thread_handle_2.take() {
        
            if let Err(e) = handle.join().map_err(|e_join| format!("ESP2 thread panicked or error on join: {:?}", e_join)) {
                self.add_esp_log_message("ESP2", e);
            }
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Source Code ");
        ui.hyperlink_to("TempSense", "https://github.com/TempSenseVR/TempSense-GUI/");
        ui.label(".");
    });
}