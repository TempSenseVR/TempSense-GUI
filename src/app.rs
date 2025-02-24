/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {

    osc_ip: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    value_max: i8,
    value_min: i8,
    osc_port: String,
    is_running: bool,
    esp_port_1: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
         
            osc_ip: "127.0.0.1".to_owned(),
            value: 2.7,
            value_max: 40,
            value_min: -10,
            osc_port: "9000".to_owned(),
            is_running: false,
            esp_port_1: "COM1".to_owned(),
        }
    }
}

impl TemplateApp {
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
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("TempSense GUI");

            ui.horizontal(|ui| {

            ui.collapsing("OSC Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("OSC IP Address: ");
                ui.add(egui::TextEdit::singleline(&mut self.osc_ip).desired_width(100.0));
            
            
                ui.label("OSC Port: ");
                ui.add(egui::TextEdit::singleline(&mut self.osc_port).desired_width(100.0));
            });
        });
        });


            ui.collapsing("Tune Temp Limits", |ui| {


            ui.horizontal(|ui| {
                ui.label("Min Temp (C): ");
                ui.add(egui::Slider::new(&mut self.value_min, -20..=45).text("Degrees"));
                
            if ui.button("-").clicked() {
                self.value_min -= 1;
            }
            if ui.button("+").clicked() {
                self.value_min += 1;
            }
        });



            
            ui.horizontal(|ui| {
            ui.label("Max Temp (C): ");

            ui.add(egui::Slider::new(&mut self.value_max, 0..=45).text("Degrees"));
            if ui.button("-").clicked() {
                self.value_max -= 1;
            }

            if ui.button("+").clicked() {
                self.value_max += 1;
            }
            });
            });

            ui.collapsing("Connect ESP", |ui| {
                ui.horizontal(|ui| {
                ui.label("ESP Port: ");
                ui.add(egui::TextEdit::singleline(&mut self.esp_port_1).desired_width(100.0));
                if ui.button("Connect").clicked() {
                    self.value_max += 1; // logic to connect to esp ig
                }
            });
            ui.horizontal(|ui| {
                ui.label("Connected: ");
                ui.label("False");
            });
            });
            

            ui.horizontal(|ui| {
                ui.label("Pelt 1:");
                ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                ui.label("ON");
                ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);
                ui.label("Temp:");
                ui.label("25C");
                ui.label("➡ 30C");
                
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
                if self.is_running{
                    ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                    ui.label("RUNNING");
                }
                else {
                    ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                    ui.label("STOPPED");
                }
                

            });
            ui.horizontal(|ui| {
            ui.label("VR Chat: ");
            ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
            ui.label("CONNECTED");
        });

            ui.separator();

            

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
