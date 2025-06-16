use std::sync::Mutex;

use crate::{
    emulator::Emulator,
    panes::{EmulatorPane, Pane, PaneDisplay, RealPane},
    theme::{self, BaseThemeChoice},
};
use egui::Theme;
use egui_dock::{AllowedSplits, DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EMULATOR: Mutex<Emulator> = Mutex::new(Emulator::new());
}
#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    pub static ref LAST_PAINT_ID: Mutex<u64> = Mutex::new(0); // this is pretty botch, more info later
}

pub fn base_to_base(
    base_in: u32,
    base_out: u32,
    input: &str,
    alphabet: impl Into<String>,
) -> String {
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
        output = alphabet.chars().next().unwrap().to_string();
    }
    output.chars().rev().collect()
}

#[derive(Default)]
struct TreeBehavior {
    added_nodes: Vec<Pane>,
    last_added: Option<(NodeIndex, SurfaceIndex)>,
}

impl TabViewer for TreeBehavior {
    type Tab = Pane;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.id)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.render(ui);
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !tab.alone
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, surface: egui_dock::SurfaceIndex, node: NodeIndex) {
        ui.set_min_width(60.0); // this is vaguely the size of the "Panes" button
        ui.style_mut().visuals.button_frame = false;

        self.add_pane_menu_items(ui, Pane::children());
        self.last_added = Some((node, surface));
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        !tab.alone
    }
}
impl TreeBehavior {
    fn add_pane_menu_items(&mut self, ui: &mut egui::Ui, pane_tree: crate::panes::PaneTree) {
        match pane_tree {
            crate::panes::PaneTree::Pane(name, pane_variant) => {
                ui.style_mut().visuals.button_frame = false;
                if ui.button(name).clicked() {
                    // Queue the pane and the target node ID for addition in the next frame
                    self.added_nodes.push(pane_variant);
                    ui.close();
                }
            }
            crate::panes::PaneTree::Children(name, children) => {
                ui.style_mut().visuals.button_frame = false;
                ui.menu_button(name, |ui| {
                    for child in children {
                        self.add_pane_menu_items(ui, child);
                    }
                });
            }
        }
    }
}

pub struct TemplateApp {
    dock_state: DockState<Pane>,
    tree_behavior: TreeBehavior,
    #[cfg(target_arch = "wasm32")]
    has_dismissed_fps: bool,
    #[cfg(target_arch = "wasm32")]
    bad_fps_score: u32,
    #[cfg(target_arch = "wasm32")]
    curr_bad_fps_prompt_open: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let span = tracing::info_span!("TemplateApp::default");
        let _guard = span.enter();

        tracing::info!("Creating new TemplateApp with comprehensive default layout");

        // Create all panes we want to include
        tracing::debug!("Creating all panes for comprehensive layout");
        let memory_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Memory(
            crate::panes::emulator::memory::MemoryPane::default(),
        ))));
        let editor_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Editor(
            crate::panes::emulator::editor::EditorPane::default(),
        ))));
        let _registers_pane = Pane::new(RealPane::EmulatorPanes(Box::new(
            EmulatorPane::Registers(crate::panes::emulator::registers::RegistersPane::default()),
        )));
        let controls_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Controls(
            crate::panes::emulator::controls::ControlsPane::default(),
        ))));
        let output_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Output(
            crate::panes::emulator::io::IoPane::default(),
        ))));
        let _cpu_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Cpu(
            crate::panes::emulator::cpu_state::CpuStatePane::default(),
        ))));
        let _help_pane = Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Help(
            crate::panes::emulator::help::HelpPane::default(),
        ))));

        let mut dock_state = DockState::new(vec![editor_pane, memory_pane]);
        let root_id = NodeIndex::root();

        dock_state
            .main_surface_mut()
            .split_left(root_id, 0.3, vec![controls_pane]);

        dock_state
            .main_surface_mut()
            .split_below(root_id, 0.7, vec![output_pane]);

        tracing::info!("TemplateApp comprehensive initialization complete");
        Self {
            dock_state,
            tree_behavior: TreeBehavior::default(),
            #[cfg(target_arch = "wasm32")]
            has_dismissed_fps: false,
            #[cfg(target_arch = "wasm32")]
            bad_fps_score: 0,
            #[cfg(target_arch = "wasm32")]
            curr_bad_fps_prompt_open: false,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let span = tracing::info_span!("TemplateApp::new");
        let _guard = span.enter();

        theme::set_global_theme(BaseThemeChoice::Dark, Some(&cc.egui_ctx));

        Default::default()
    }

    // // Helper function to recursively build the pane menu
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let update_span = tracing::info_span!("TemplateApp::update");
        let _update_guard = update_span.enter();

        #[cfg(target_arch = "wasm32")]
        if !self.has_dismissed_fps {
            let fps = ctx.input(|i| i.stable_dt);
            if fps < 50.0 {
                self.bad_fps_score += 1;
            } else {
                self.bad_fps_score -= 1;
            }

            if self.bad_fps_score >= 300 {
                self.curr_bad_fps_prompt_open = true;
            }
        }

        #[cfg(target_arch = "wasm32")]
        if self.curr_bad_fps_prompt_open {
            egui::Window::new("Bad fps detected").collapsible(false).show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    use egui::{Hyperlink, RichText};

                    ui.label("It seems you have bad fps on the web version of the tool. The desktop version is likely to run far better. You can find downloads");
                    ui.add(Hyperlink::from_label_and_url(RichText::new("here").strong(), "https://github.com/JackCrumpLeys/textbook210_emulator/releases/tag/main").open_in_new_tab(true));
                    ui.label(".");
                });
                ui.separator();
                ui.horizontal_top(|ui| {
                    if ui.button("Ok").clicked() {
                        self.curr_bad_fps_prompt_open = false;
                        self.has_dismissed_fps = true;
                    }
                })
            });
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            #[allow(deprecated)] // idk what egui is on about here
            egui::menu::bar(ui, |ui| {
                // File Menu (standard)
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                // Windows Menu (layout reset)
                ui.menu_button("Windows", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        tracing::info!("Resetting layout to default");
                        *self = Self::default(); // Reset the entire app state
                    }
                    // You could add more layout options here
                });

                // UI Menu (scaling, theme)
                ui.menu_button("UI", |ui| {
                    // slider for ui scale
                    let mut scale = ctx.zoom_factor();
                    let res = ui.add(egui::Slider::new(&mut scale, 0.5..=5.0).text("UI Scale"));
                    if !res.dragged() && res.changed() {
                        tracing::info!("Setting new UI scale: {}", scale);
                        ctx.set_zoom_factor(scale);
                    }
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        let curr_theme = match ctx.theme() {
            Theme::Light => BaseThemeChoice::Light,
            Theme::Dark => BaseThemeChoice::Dark,
        };
        if theme::CURRENT_THEME_SETTINGS.lock().unwrap().base_theme != curr_theme {
            theme::set_global_theme(curr_theme, Some(ctx));
        }

        egui::CentralPanel::default().show(ctx, |_ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
        });

        self.dock_state.iter_surfaces_mut().for_each(|sur| {
            sur.iter_nodes_mut().for_each(|n| {
                if n.is_leaf() {
                    let tabs_mut = n.tabs_mut().unwrap();
                    if tabs_mut.len() == 1 {
                        tabs_mut[0].alone = true;
                    } else {
                        for t in tabs_mut {
                            t.alone = false
                        }
                    }
                }
            });
        });

        DockArea::new(&mut self.dock_state)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show_leaf_close_all_buttons(false)
            .draggable_tabs(false)
            .style(Style::from_egui(ctx.style().as_ref()))
            .allowed_splits(AllowedSplits::None)
            .show(ctx, &mut self.tree_behavior);

        if let Some((nodei, sur)) = self.tree_behavior.last_added {
            self.tree_behavior.added_nodes.drain(..).for_each(|node| {
                self.dock_state.set_focused_node_and_surface((sur, nodei));
                self.dock_state.push_to_focused_leaf(node);
            });
        }

        // why do we need this? Well our update loop cannot get the egui context so cannot
        // see the pass number, we need this to request a repaint if the emulator state
        // changes.
        #[cfg(not(target_arch = "wasm32"))]
        {
            *LAST_PAINT_ID.lock().unwrap() = ctx.cumulative_pass_nr_for(egui::ViewportId::ROOT);
        }
        #[cfg(target_arch = "wasm32")]
        ctx.request_repaint(); // I could not find a way to repaint on change on the wasm backend without forking eframe
    }
}
