/// We derive Deserialize/Serialize so we can persist app state on shutdown.

use std::time::Duration;
use std::sync::mpsc::{self, Receiver};

// Define the different pages of the application
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
    pub is_running: bool,
    pub esp_port_1: String,
    pub pelt_temp_1: i8,
    #[serde(skip)]
    pub osc_receiver: Receiver<i8>, // Channel receiver for OSC updates
    #[serde(skip)]
    pub last_update_time: std::time::Instant,
    #[serde(skip)]
    pub current_page: Page, // Track the current page
}

impl Default for TemplateApp {
    fn default() -> Self {
        let (_, osc_receiver) = mpsc::channel(); // Create a channel
        Self {
            osc_ip: "127.0.0.1".to_owned(),
            value: 2.7,
            value_max: 40,
            value_min: -10,
            osc_port: "9000".to_owned(),
            is_running: false,
            esp_port_1: "COM1".to_owned(),
            pelt_temp_1: 0,
            last_update_time: std::time::Instant::now(),
            osc_receiver, // Initialize the receiver
            current_page: Page::Home, // Default to home page
        }
    }
}

impl TemplateApp {
    pub fn update_pelt_temp(&mut self, _id: i8, temp: i8) {
        self.pelt_temp_1 = temp;
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    // Render the Home page content
    fn render_home_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Pelt 1:");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ui.label("ON");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
            ui.label("Temp:");
            ui.label("25C");
            ui.label("➡ ");
            ui.label(format!("{}°C", self.pelt_temp_1));
        //    if ui.button("Simulate Update").clicked() {
          //      self.update_pelt_temp(1, 25); // Example temperature
            // }
        });
        
        ui.horizontal(|ui| {
            ui.label("Pelt 2:");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ui.label("ON");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
            ui.label("Temp:");
            ui.label("23C");
            ui.label("➡ 30C");
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("START ▶").clicked() {
                self.is_running = true;
            }
            if ui.button("STOP ALL ■").clicked() {
                self.is_running = false;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Status:");
            if self.is_running {
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("RUNNING");
            } else {
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("STOPPED");
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("OSC: ");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ui.label("CONNECTED");
        });
    }

    // Render the OSC Settings page content
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
            // Add logic to apply OSC settings
            ui.label("Settings Applied");
        }
        
        ui.add_space(10.0);
        
        // Status display
        ui.horizontal(|ui| {
            ui.label("OSC Status:");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ui.label("CONNECTED");
        });
    }

    // Render the ESP Connection page content
    fn render_esp_connection_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("ESP Connection");
        
        ui.horizontal(|ui| {
            ui.label("ESP Port:");
            ui.add(egui::TextEdit::singleline(&mut self.esp_port_1).desired_width(150.0));
        });
        
        ui.add_space(10.0);
        
        if ui.button("Connect ESP").clicked() {
            self.value_max += 1; // logic to connect to esp ig
        }
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Connected:");
            ui.label("False");
        });
        
        ui.add_space(20.0);
        
        // Add more ESP-specific settings and controls here
        ui.collapsing("Advanced ESP Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Baud Rate:");
                ui.label("115200");
            });
            
            ui.horizontal(|ui| {
                ui.label("Timeout (ms):");
                ui.label("1000");
            });
        });
    }
    fn render_app_settings_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("App Settings");
        ui.separator();
        
        egui::widgets::global_theme_preference_buttons(ui);
    
    }

}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any incoming OSC messages
        if let Ok(temp) = self.osc_receiver.try_recv() {
            self.update_pelt_temp(1, temp); // Update the temperature
            ctx.request_repaint(); // Force the GUI to refresh
        }

        ctx.request_repaint_after_for(Duration::from_millis(750), ctx.viewport_id());

        // Top panel with menu and page navigation
      //  egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
          //  egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
             //   let is_web = cfg!(target_arch = "wasm32");
               // if !is_web {
                  //  ui.menu_button("File", |ui| {
                   //     if ui.button("Quit").clicked() {
                     //       ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                       // }
                 //   });
                 //   ui.add_space(16.0);
               // }

                
        //    });
      //  });
        
        // Page navigation buttons at the top center
        egui::TopBottomPanel::top("page_navigation").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
              //  ui.add_space(3.0); // Add padding at the top
                
                let button_height = 32.0;
                let button_width = 100.0;
 
                // Create a horizontal layout with centered content
                ui.horizontal_centered(|ui| {
                    
                    // Increase spacing between buttons
                    ui.spacing_mut().item_spacing.x = 5.0;
                    ui.spacing_mut().button_padding = egui::vec2(0.0, 8.0);
                    
                    // Create custom sized buttons
                    if ui.add_sized(
                        [button_width, button_height],
                        egui::SelectableLabel::new(self.current_page == Page::Home, "Home")
                    ).clicked() {
                        self.current_page = Page::Home;
                    }
                    
                    if ui.add_sized(
                        [button_width, button_height],
                        egui::SelectableLabel::new(self.current_page == Page::OscSettings, "OSC Settings")
                    ).clicked() {
                        self.current_page = Page::OscSettings;
                    }
                    
                    if ui.add_sized(
                        [button_width, button_height],
                        egui::SelectableLabel::new(self.current_page == Page::EspConnection, "ESP Connection")
                    ).clicked() {
                        self.current_page = Page::EspConnection;
                    }
                    
                    if ui.add_sized(
                        [button_width, button_height],
                        egui::SelectableLabel::new(self.current_page == Page::AppSettings, "App Settings")
                    ).clicked() {
                        self.current_page = Page::AppSettings;
                    }
                });
                
                ui.add_space(8.0); // Add padding at the bottom
            });
        });
        // Central panel with the current page content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Display different content based on the current page
            match self.current_page {
                Page::Home => self.render_home_page(ui),
                Page::OscSettings => self.render_osc_settings_page(ui),
                Page::EspConnection => self.render_esp_connection_page(ui),
                Page::AppSettings => self.render_app_settings_page(ui),
            }

            ui.separator();

            // Bottom section with credits
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Source Code ");
        ui.hyperlink_to("TempSense", "https://github.com/emilk/egui");
        ui.label(".");
    });
}