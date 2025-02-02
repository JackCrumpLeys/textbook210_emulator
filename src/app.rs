use std::{any::Any, collections::BTreeSet, str::FromStr};

use eframe::glow::LINEAR;
use egui::{ahash::HashMap, Label, RichText};
use egui_extras::Column;
use egui_tiles::{Behavior, SimplificationOptions, Tree};

use crate::emulator::Emulator;

pub trait Window: Default {
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Pane {
    BaseConverter(BaseConverter),
    Emulator(EmulatorPane),
}

#[derive(Debug, Clone, Default)]
pub struct BaseConverter {
    input: String,
    output_hist: Vec<String>,
    alphabet: String,
    base_in: u32,
    base_out: u32,
    case_sensitive: bool,
    uppercase: bool,
}

impl Window for BaseConverter {
    fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("This is a base converter. Enter a number, select the input and output bases, adjust the alphabet, and click 'Convert' to see the result. You can also toggle case sensitivity and choose between uppercase and lowercase conversion.");

        if self.case_sensitive {
            ui.label(RichText::new("⚠ Note: Case sensitivity is enabled. ⚠")
                            .small()
                            .color(ui.visuals().warn_fg_color)).on_hover_text("Case sensitivity is enabled.  You can change this behavior by toggling the 'Case Sensitive' checkbox.");
        }
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.input);
            ui.label("->");
            if let Some(most_recent_output) = self.output_hist.last() {
                ui.label(most_recent_output);
            } else {
                ui.label("");
            }
            if ui.button("Convert").clicked() {
                // Call the stub function base_to_base
                if !self.case_sensitive {
                    if self.uppercase {
                        self.input = self.input.to_uppercase();
                    } else {
                        self.input = self.input.to_lowercase();
                    }
                }
                let output = base_to_base(self.base_in, self.base_out, &self.input, &self.alphabet);
                self.output_hist.push(output);
            }
        });

        ui.horizontal(|ui| {
            ui.label("Alphabet:");
            ui.text_edit_singleline(&mut self.alphabet);
        });

        let max_base = self.alphabet.len() as u32;

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.base_in, 2..=max_base));
            ui.add(egui::Slider::new(&mut self.base_out, 2..=max_base));
        });

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.case_sensitive, "Case Sensitive");
            if !self.case_sensitive {
                ui.checkbox(&mut self.uppercase, "Uppercase");
            }
        });

        ui.separator();

        egui::CollapsingHeader::new("History")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for line in self.output_hist.iter() {
                        ui.label(line);
                    }
                });
            });
    }

    fn title(&self) -> String {
        "Base Converter".to_owned()
    }
}

fn base_to_base(base_in: u32, base_out: u32, input: &str, alphabet: impl Into<String>) -> String {
    let alphabet: String = alphabet.into();
    let mut output = String::new();
    let mut num = 0;
    let mut place = 1;
    for c in input.chars().rev() {
        let digit = match alphabet.find(c) {
            Some(d) => d as u32,
            None => {
                return "Invalid input".to_owned();
            }
        };
        num += digit * place;
        place *= base_in;
    }
    while num > 0 {
        let digit = num % base_out;
        num /= base_out;
        let c = match alphabet.chars().nth(digit as usize) {
            Some(c) => c,
            None => {
                return "Invalid input".to_owned();
            }
        };
        output.push(c);
    }
    if output == String::new() {
        output = alphabet.chars().nth(0).unwrap().to_string();
    }
    output.chars().rev().collect()
}

#[derive(Debug, Clone, Default)]
struct EmulatorPane {
    input: i16,
    output: String,
    program: String,
    last_compiled: String,
    breakpoints: Vec<usize>,
    error: Option<(String, usize)>,
    emulator: Emulator,
    running: bool,
    line_to_address: HashMap<usize, usize>,
    show_machine_code: bool,
    speed: u32,
}

impl Window for EmulatorPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("This is an emulator. Enter a program and click 'compile' to attempt loading the code");

            // TEXT EDITOR
            egui_code_editor::CodeEditor::default()
                .with_syntax(
                    egui_code_editor::Syntax::new("110_textbook")
                        .with_comment("--")
                        .with_keywords(BTreeSet::from([
                            "LOAD",
                            "STORE",
                            "ADD",
                            "SUBTRACT",
                            "INCREMENT",
                            "DECREMENT",
                            "JUMP",
                            "JUMPGT",
                            "JUMPEQ",
                            "JUMPLT",
                            "IN",
                            "OUT",
                            "HALT",
                            "CLEAR",
                            "JUMPNEQ",
                            "COMPARE",
                        ]))
                        .with_special(BTreeSet::from([":", ".data", ".begin", ".end"]))
                        .with_case_sensitive(false),
                )
                .with_theme(egui_code_editor::ColorTheme::SONOKAI)
                .vscroll(true)
                .show(ui, &mut self.program);

            ui.add(egui::Slider::new(&mut self.speed, 1..=100));

            // BUTTONS
            // STEP - RUN - RESET - SUBMIT INPUT
            ui.horizontal(|ui| {
                if ui.button("Step").clicked() {
                    self.emulator.step();
                }
                if self.running {
                    if !self.emulator.await_input.is_some() {
                        let mut i = 0;
                        while self.running && i < self.speed {
                            self.running = self.emulator.step();
                            i +=1
                        }
                        if self.breakpoints.contains(&(self.emulator.pc as usize)) {
                            self.running = false;
                        }
                    }
                    if ui.button("Pause").clicked() {
                        self.running = false;
                    }
                } else if ui.button("Run").clicked() {
                    self.running = true;
                }

                if self.emulator.memory.iter().all(|x| *x == 0) {
                    // compile
                    if ui.button("Compile").clicked() {
                        let data_to_load = Emulator::parse_program(&self.program);
                        if let Ok(data_to_load) = data_to_load {
                            self.line_to_address = data_to_load.0.iter().enumerate().map(|(i, (x,_))| (*x,i)).collect();
                            self.emulator.flash_memory(data_to_load.0.into_iter().map(|(x,y)| y).collect());
                            self.error = None;
                        } else {
                            self.error = Some(data_to_load.unwrap_err());
                        }
                        self.last_compiled = self.program.clone();
                    }
                } else if ui.button("Reset & compile").clicked() {
                    self.emulator = Emulator::new();
                    let data_to_load = Emulator::parse_program(&self.program);
                    if let Ok(data_to_load) = data_to_load {
                        self.line_to_address = data_to_load.0.iter().enumerate().map(|(i, (x,_))| (*x,i)).collect();
                        self.emulator.flash_memory(data_to_load.0.into_iter().map(|(x,y)| y).collect());
                        self.error = None;
                    } else {
                        self.error = Some(data_to_load.unwrap_err());
                    }
                    self.last_compiled = self.program.clone();
                }
                ui.separator();

                if self.emulator.await_input.is_some() {
                    ui.add(
                        egui::DragValue::new(&mut self.input)
                            .range(i16::MIN..=i16::MAX)
                            .speed(1.0),
                    );
                    if ui.button("Submit Input").clicked() {
                        self.emulator.set_input(self.input as u16);
                        if self.running {
                            self.running = self.emulator.step();
                        }
                    }
                }

            });
            // STATE
            // r - pc - gt - lt - eq
            ui.horizontal(|ui| {
                ui.label(format!("r: {}", self.emulator.r));
                ui.label(format!("pc: {}", self.emulator.pc));
                ui.label(format!("gt: {}", self.emulator.gt));
                ui.label(format!("lt: {}", self.emulator.lt));
                ui.label(format!("eq: {}", self.emulator.eq));
            });

            ui.separator();

            egui::CollapsingHeader::new("Output")
                .default_open(true)
                .show(ui, |ui| {
                    for line in self.emulator.get_output().iter() {
                        ui.label(line.to_string());
                    }
                });

            if let Some((error, line)) = &self.error {
                ui.label(
                    RichText::new(format!("Error on line {}: {}", line, error))
                        .small()
                        .color(ui.visuals().warn_fg_color),
                );
            }

            ui.separator();

            // tick box for machine code

            ui.checkbox(&mut self.show_machine_code, "Show Machine Code");

            let mut  longest_label  = 0;
            let mut  longest_body   = 0;
            let mut  longest_operand = 0;
            for (i, line )in self.last_compiled.lines().enumerate() {
                if !self.line_to_address.contains_key(&i) || line == "" {
                    continue;
                }
                // unwrap is allg becuase we alrady returned on empty string
                let mut split = line.split("--").next().unwrap().split(' ').filter(|s| *s!="");
                let (mut len_lab, mut len_body, mut len_op) = (0,0,0);
                match split.clone().count() {
                    0 => {
                        // idk how we would get here but I dont wanna crash the program over it
                        log::error!("we somhow got a line with no content though the compiler");
                    },
                    1 => {
                        len_lab = split.next().unwrap().len();
                    },
                    2 => {
                        let first = split.next().unwrap();

                        if first.chars().last().unwrap() == ':' {
                            len_lab = first.len();
                            len_body = split.next().unwrap().len();
                        } else {
                            len_body= first.len();
                            len_lab  = split.next().unwrap().len();
                        }

                    },
                    3 => {
                        len_body = split.next().unwrap().len();
                        len_lab = split.next().unwrap().len();
                        len_op = split.next().unwrap().len();
                    },
                    _ => {

                    }
                }
                if len_lab > longest_label {
                    longest_label = len_lab;
                }
                if len_body > longest_body {
                    longest_body = len_body;
                }
                if len_op> longest_operand{
                    longest_operand= len_op;
                }
            }
            log::info!("longest label: {}, longest body: {}, longest operand: {}", longest_label, longest_body, longest_operand);



            // either display the output with error annotations or display the output with pc annotation and breakpoints as well as translations for all the instructions
            for (i, line) in self.last_compiled.lines().enumerate() {
                let line = line.trim().to_string().to_ascii_uppercase();
                let mut label = line.clone();
                if let Some((error, line)) = &self.error {
                    if *line == i {
                        label = format!("{} (error: {})", label, error);
                    }
                }

                if self.breakpoints.contains(&i) {
                    label = format!("{} (breakpoint)", label);
                }
                if label.contains(".DATA") {
                    label = format!("{}", label.split(".").next().unwrap());
                }


                if let Some(address) = self.line_to_address.get(&(i)) {
                    // alaign the labels based on content

                    let label_parts: Vec<&str> = line.split("--").next().unwrap().split_whitespace().collect();
                    let formatted_label = match label_parts.len() {
                        1 => format!("{:<width1$} {:<width2$}", "", label_parts[0], width1 = longest_label, width2 = longest_body),
                        2 => {
                            if label_parts[0].ends_with(':') {
                                format!("{:<width1$} {:<width2$}", label_parts[0], label_parts[1], width1 = longest_label, width2 = longest_body)
                            } else {
                                format!("{:<width1$} {:<width2$}", "", label_parts[0], width1 = longest_label, width2 = longest_body) + &format!(" {:<width$}", label_parts[1], width = longest_operand)
                            }
                        },
                        3 => format!("{:<width1$} {:<width2$} {:<width3$}", label_parts[0], label_parts[1], label_parts[2], width1 = longest_label, width2 = longest_body, width3 = longest_operand),
                        _ => format!("{:<width1$} {:<width2$} {:<width3$} {}", label_parts[0], label_parts[1], label_parts[2],label_parts[4..].join(" "), width1 = longest_label, width2 = longest_body, width3 = longest_operand),
                    };
                    label = formatted_label;
                    log::info!("label: {}", label);



                    if self.show_machine_code {
                        if let Some(instruction) = self.emulator.memory.get(*address) {
                            label = format!("{:016b}", instruction);
                        }
                    }
                    if *address == self.emulator.pc as usize {
                        label = format!("{}: {} (pc)", address, label);
                    } else {
                        label = format!("{}: {}", address, label);
                    }
                    ui.horizontal(|ui| {
                        if ui.button("🛑").clicked() {
                            if self.breakpoints.contains(&address) {
                                self.breakpoints.retain(|x| *x != *address);
                            } else {
                                self.breakpoints.push(*address);
                            }
                        }
                        // green if pc
                        // orange if error
                        // light red if breakpoint
                        // light blue if data
                        // light purple if label
                        // normal if none
                        if let Some(instruction) = self.emulator.memory.get(*address) {
                            if self.emulator.pc as usize == *address {
                                ui.label(RichText::new(label).background_color(egui::Color32::GREEN).color(egui::Color32::BLACK).monospace());
                            } else if self.error.as_ref().map_or(false, |(_, line)| *line == *address) {
                                ui.label(RichText::new(label).background_color(egui::Color32::ORANGE).color(egui::Color32::BLACK).monospace());
                            } else if self.breakpoints.contains(address) {
                                ui.label(RichText::new(label).background_color(egui::Color32::LIGHT_RED).color(egui::Color32::BLACK).monospace());
                            } else if line.to_ascii_lowercase().contains(".DATA") {
                                ui.label(RichText::new(label).background_color(egui::Color32::LIGHT_BLUE).color(egui::Color32::BLACK).monospace());
                            } else {
                                ui.label(RichText::new(label).monospace());
                            }
                        }

                            if line.contains(".DATA") {
                                let mut value = self.emulator.memory[*address] as i16;
                                ui.add(
                                    egui::DragValue::new(&mut value)
                                );
                                self.emulator.memory[*address] = value as u16;
                            }

                    });
                }else if self.error.as_ref().map_or(false, |(_, line)| *line == i) {
                                ui.label(RichText::new(label).background_color(egui::Color32::ORANGE).color(egui::Color32::BLACK).monospace());
                            }
else {
                ui.label(label);
            }

            }         });
    }

    fn title(&self) -> String {
        "Emulator".to_owned()
    }
}

impl Default for Pane {
    fn default() -> Self {
        Pane::BaseConverter(BaseConverter {
            input: "0".to_owned(),
            output_hist: Vec::new(),
            alphabet: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_owned(),
            base_in: 10,
            base_out: 2,
            case_sensitive: true,
            uppercase: true,
        })
    }
}

impl From<Pane> for String {
    fn from(pane: Pane) -> String {
        match pane {
            Pane::BaseConverter(a) => a.title(),
            Pane::Emulator(a) => a.title(),
        }
    }
}

impl Pane {
    fn to_string(&self) -> String {
        String::from(self.clone())
    }

    fn render(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) {
        match self {
            Pane::BaseConverter(a) => a.render(ui),
            Pane::Emulator(a) => a.render(ui),
        };
    }

    fn iter_default() -> impl Iterator<Item = Pane> {
        vec![
            Pane::BaseConverter(BaseConverter::default()),
            Pane::Emulator(EmulatorPane::default()),
        ]
        .into_iter()
    }
}

#[derive(Default)]
struct TreeBehavior {
    add_child_to: Option<(egui_tiles::TileId, Pane)>,
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("{}", pane.to_string()).into()
    }

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<Pane>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        true
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        // Give each pane a unique color:

        pane.render(ui, tile_id);
        egui_tiles::UiResponse::None
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        egui::ComboBox::from_label("")
            .selected_text("➕")
            .show_ui(ui, |ui| {
                for pane in Pane::iter_default() {
                    if ui.button(pane.to_string()).clicked() {
                        self.add_child_to = Some((tile_id, pane));
                    }
                }
            });
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)]
    value: f32,
    #[serde(skip)]
    tree: egui_tiles::Tree<Pane>,
    #[serde(skip)]
    tree_behavior: TreeBehavior,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut next_view_nr = 0;
        let mut gen_pane = || {
            let pane = Pane::default();
            next_view_nr += 1;
            pane
        };

        let mut tiles = egui_tiles::Tiles::default();

        let mut tabs = vec![];
        tabs.push(tiles.insert_pane(gen_pane()));

        let root = tiles.insert_tab_tile(tabs);

        let tree = egui_tiles::Tree::new("my_tree", root, tiles);

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            tree,
            tree_behavior: TreeBehavior::default(),
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

        if let Some((tile_id, pane)) = self.tree_behavior.add_child_to.take() {
            let new_pane = self.tree.tiles.insert_pane(pane);
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                self.tree.tiles.get_mut(tile_id)
            {
                let new_tile_id = tabs.add_child(new_pane);
                tabs.set_active(new_pane);
            }
        }

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

                ui.menu_button("windows", |ui| {
                    if ui.button("Reset layout").clicked() {
                        self.tree = Self::default().tree;
                    }
                });

                ui.menu_button("ui", |ui| {
                    // slider for ui scale
                    let mut scale = ctx.zoom_factor();
                    let dragging = ui
                        .add(egui::Slider::new(&mut scale, 0.5..=5.0).text("UI scale"))
                        .dragged();
                    if !dragging {
                        ctx.set_zoom_factor(scale);
                    }
                });

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            self.tree.ui(&mut self.tree_behavior, ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

        ctx.request_repaint()
    }
}
