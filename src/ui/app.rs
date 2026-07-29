use eframe::egui;
use crate::core::molecule::{Molecule, BondOrder, Atom};
use crate::core::smiles::generate_smiles;
use super::canvas::draw_canvas;

#[derive(PartialEq, Clone)]
pub enum Tool {
    Select,
    Erase,
    Atom(&'static str),
    Bond(BondOrder),
}

pub struct OpenSmilesApp {
    pub molecule: Molecule,
    pub current_tool: Tool,
    pub smiles: String,
    
    // UI state
    pub dragging_from: Option<usize>,
    pub hovered_atom: Option<usize>,
    pub hovered_bond: Option<usize>,
    
    // Canvas transform
    pub zoom: f32,
    pub pan: egui::Vec2,
    
    // Settings
    pub show_settings: bool,
    pub show_periodic_table: bool,
    pub atom_colors: std::collections::HashMap<String, String>,
    
    // OCR State
    pub ocr_result: std::sync::Arc<std::sync::Mutex<Option<Result<String, String>>>>,
    pub is_processing_ocr: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl Default for OpenSmilesApp {
    fn default() -> Self {
        let mut atom_colors = std::collections::HashMap::new();
        atom_colors.insert("O".to_string(), "#D22828".to_string());
        atom_colors.insert("N".to_string(), "#2828D2".to_string());
        atom_colors.insert("S".to_string(), "#C8C800".to_string());
        atom_colors.insert("P".to_string(), "#FFA500".to_string());
        atom_colors.insert("Halogen".to_string(), "#28D228".to_string());
        atom_colors.insert("H".to_string(), "#787878".to_string());
        atom_colors.insert("Default".to_string(), "#646464".to_string());

        Self {
            molecule: Molecule::new(),
            current_tool: Tool::Atom("C"),
            smiles: String::new(),
            dragging_from: None,
            hovered_atom: None,
            hovered_bond: None,
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            show_settings: false,
            show_periodic_table: false,
            atom_colors,
            ocr_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            is_processing_ocr: std::sync::Arc::new(std::sync::Mutex::new(false)),
        }
    }
}

impl eframe::App for OpenSmilesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle OCR Result
        if let Ok(mut res_lock) = self.ocr_result.try_lock() {
            if let Some(res) = res_lock.take() {
                match res {
                    Ok(smiles) => {
                        self.smiles = smiles;
                        if let Some(mut mol) = crate::core::smiles::parse_smiles(&self.smiles) {
                            crate::core::layout::generate_2d_layout(&mut mol);
                            self.molecule = mol;
                            self.pan = egui::Vec2::ZERO;
                            self.zoom = 1.0;
                        }
                    }
                    Err(e) => {
                        eprintln!("OCR Error: {}", e);
                        if e == "ERROR_NOT_INSTALLED" {
                            self.smiles = "ERROR: Please run `pip install decimer` to use OCR".to_string();
                        } else {
                            self.smiles = format!("OCR ERROR: {}", e);
                        }
                    }
                }
            }
        }

        // Apply Open DoE theme
        let mut visuals = egui::Visuals::light();
        let coral_red = egui::Color32::from_rgb(255, 91, 91);
        let dark_text = egui::Color32::from_rgb(74, 74, 74);
        let light_border = egui::Color32::from_rgb(230, 230, 235);
        let app_bg = egui::Color32::from_rgb(248, 249, 250);
        
        visuals.widgets.noninteractive.bg_fill = app_bg;
        visuals.panel_fill = egui::Color32::WHITE;
        visuals.selection.bg_fill = coral_red;
        visuals.window_rounding = egui::Rounding::same(6.0);
        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::Vec2::new(12.0, 12.0);
        style.spacing.button_padding = egui::Vec2::new(10.0, 6.0);
        ctx.set_style(style);

        // Top panel for SMILES output
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::default()
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, light_border))
                .inner_margin(12.0))
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("SMILES:").strong().size(16.0).color(dark_text));
                ui.add_space(8.0);
                
                // Styled TextEdit
                let text_edit = egui::TextEdit::singleline(&mut self.smiles)
                    .desired_width(ui.available_width() - 200.0)
                    .font(egui::TextStyle::Monospace)
                    .margin(egui::Vec2::new(8.0, 6.0));
                
                ui.add(text_edit);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("Settings").color(dark_text))
                        .fill(egui::Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, light_border))
                        .rounding(4.0)).clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    ui.add_space(8.0);
                    // Secondary Button: Copy
                    if ui.add(egui::Button::new(egui::RichText::new("Copy").color(dark_text))
                        .fill(egui::Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, light_border))
                        .rounding(4.0)).clicked() {
                        ui.output_mut(|o| o.copied_text = self.smiles.clone());
                    }
                });
            });
        });

        // Settings Window
        if self.show_settings {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Atom Colors (Hex format: #RRGGBB)").strong().color(dark_text));
                    ui.add_space(8.0);
                    let keys = ["O", "N", "S", "P", "Halogen", "H", "Default"];
                    for k in keys {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:>7}:", k));
                            if let Some(color_str) = self.atom_colors.get_mut(k) {
                                ui.text_edit_singleline(color_str);
                            }
                        });
                    }
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
        }

        // Periodic Table Window
        if self.show_periodic_table {
            egui::Window::new("Periodic Table")
                .collapsible(false)
                .resizable(false)
                .default_width(400.0)
                .show(ctx, |ui| {
                    let all_elements = [
                        "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne",
                        "Na", "Mg", "Al", "Si", "P", "S", "Cl", "Ar",
                        "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr",
                        "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe",
                        "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn",
                        "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh", "Fl", "Mc", "Lv", "Ts", "Og"
                    ];
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        for &el in &all_elements {
                            let is_selected = self.current_tool == Tool::Atom(el);
                            let btn = egui::Button::new(egui::RichText::new(el).color(if is_selected { egui::Color32::WHITE } else { dark_text }))
                                .fill(if is_selected { coral_red } else { egui::Color32::WHITE })
                                .stroke(egui::Stroke::new(1.0, light_border));
                            if ui.add_sized([32.0, 32.0], btn).clicked() {
                                self.current_tool = Tool::Atom(el);
                                self.show_periodic_table = false;
                            }
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.show_periodic_table = false;
                    }
                });
        }

        // Left panel for tools
        egui::SidePanel::left("left_panel")
            .exact_width(240.0)
            .frame(egui::Frame::default()
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(1.0, light_border))
                .inner_margin(16.0))
            .show(ctx, |ui| {
            
            // Bottom buttons (fixed at bottom)
            egui::TopBottomPanel::bottom("left_panel_bottom")
                .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(0.0, 8.0)))
                .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    
                    // Secondary Action Button: Clear Canvas
                    let clear_btn = egui::Button::new(egui::RichText::new("Clear Canvas").color(dark_text))
                        .fill(egui::Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, light_border))
                        .rounding(6.0);
                    
                    if ui.add_sized([ui.available_width(), 32.0], clear_btn).clicked() {
                        self.molecule.clear();
                        self.smiles.clear();
                        self.pan = egui::Vec2::ZERO;
                        self.zoom = 1.0;
                    }
                    
                    ui.add_space(8.0);
                    
                    // OCR Button
                    let is_processing = *self.is_processing_ocr.lock().unwrap();
                    let ocr_btn_text = if is_processing {
                        "Processing OCR..."
                    } else {
                        "OCR Image to Structure"
                    };
                    let ocr_btn = egui::Button::new(egui::RichText::new(ocr_btn_text).color(dark_text))
                        .fill(egui::Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, light_border))
                        .rounding(6.0);
                    
                    if ui.add_sized([ui.available_width(), 32.0], ocr_btn).clicked() && !is_processing {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Image", &["png", "jpg", "jpeg"])
                            .pick_file() {
                            
                            let ocr_res_clone = std::sync::Arc::clone(&self.ocr_result);
                            let is_proc_clone = std::sync::Arc::clone(&self.is_processing_ocr);
                            *is_proc_clone.lock().unwrap() = true;
                            
                            std::thread::spawn(move || {
                                let path_str = path.to_string_lossy().to_string();
                                
                                // Prefer virtual environment python if it exists
                                let mut python_bin = "python3".to_string();
                                if std::path::Path::new(".venv/bin/python3").exists() {
                                    python_bin = ".venv/bin/python3".to_string();
                                } else if std::path::Path::new(".venv/Scripts/python.exe").exists() {
                                    python_bin = ".venv/Scripts/python.exe".to_string();
                                }
                                
                                let output = std::process::Command::new(&python_bin)
                                    .arg("ocr_backend.py")
                                    .arg(&path_str)
                                    .output();
                                
                                let mut res = Err("Failed to execute python3".to_string());
                                if let Ok(out) = output {
                                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                                    
                                    if out.status.success() && !stdout.is_empty() {
                                        res = Ok(stdout);
                                    } else {
                                        res = Err(if !stderr.is_empty() { stderr } else { "Unknown error".to_string() });
                                    }
                                }
                                
                                *ocr_res_clone.lock().unwrap() = Some(res);
                                *is_proc_clone.lock().unwrap() = false;
                            });
                        }
                    }
                    
                    // Primary Action Button 2: Clean Structure
                    let clean_btn = egui::Button::new(egui::RichText::new("Clean Structure").strong().color(egui::Color32::WHITE))
                        .fill(coral_red)
                        .rounding(6.0);
                    
                    if ui.add_sized([ui.available_width(), 36.0], clean_btn).clicked() {
                        crate::core::layout::generate_2d_layout(&mut self.molecule);
                        self.pan = egui::Vec2::ZERO;
                    }

                    ui.add_space(8.0);
                    
                    // Primary Action Button: Generate Design
                    let generate_btn = egui::Button::new(egui::RichText::new("Generate Design").strong().color(egui::Color32::WHITE))
                        .fill(coral_red)
                        .rounding(6.0);
                    
                    if ui.add_sized([ui.available_width(), 36.0], generate_btn).clicked() {
                        if let Some(mut mol) = crate::core::smiles::parse_smiles(&self.smiles) {
                            crate::core::layout::generate_2d_layout(&mut mol);
                            self.molecule = mol;
                            self.pan = egui::Vec2::ZERO;
                            self.zoom = 1.0;
                        }
                    }
                    
                    ui.add_space(16.0);
                });
            });

            // The rest goes in a ScrollArea
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(egui::RichText::new("DESIGN CONFIGURATION").strong().size(12.0).color(egui::Color32::GRAY));
                ui.add_space(8.0);
                
                let group_frame = egui::Frame::default()
                    .fill(egui::Color32::WHITE)
                    .stroke(egui::Stroke::new(1.0, light_border))
                    .rounding(6.0)
                    .inner_margin(12.0);

                ui.label(egui::RichText::new("Basic Tools").strong().color(dark_text));
                group_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.radio_value(&mut self.current_tool, Tool::Select, "Select / Move");
                    ui.radio_value(&mut self.current_tool, Tool::Erase, "Erase");
                });
                
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Bonds").strong().color(dark_text));
                group_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.radio_value(&mut self.current_tool, Tool::Bond(BondOrder::Single), "Single Bond");
                    ui.radio_value(&mut self.current_tool, Tool::Bond(BondOrder::Double), "Double Bond");
                    ui.radio_value(&mut self.current_tool, Tool::Bond(BondOrder::Triple), "Triple Bond");
                });
                
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Atoms").strong().color(dark_text));
                group_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let elements = [
                        ("C", "Carbon (Grey)"), ("N", "Nitrogen (Blue)"), 
                        ("O", "Oxygen (Red)"), ("S", "Sulfur (Yellow)"), 
                        ("P", "Phosphorus (Orange)"), ("F", "Fluorine (Green)"), 
                        ("Cl", "Chlorine (Green)"), ("Br", "Bromine (Green)"), 
                        ("I", "Iodine (Green)")
                    ];
                    for &(el, name) in &elements {
                        ui.radio_value(&mut self.current_tool, Tool::Atom(el), name);
                    }
                    
                    ui.add_space(4.0);
                    if ui.button("More Elements...").clicked() {
                        self.show_periodic_table = true;
                    }
                });
            });
        });

        // Central canvas
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(app_bg).inner_margin(0.0))
            .show(ctx, |ui| {
            let changed = draw_canvas(ui, self);
            if changed {
                self.smiles = generate_smiles(&self.molecule);
            }
        });
    }
}
