use std::sync::Mutex;

use egui_tiles::SimplificationOptions;

use crate::{
    emulator::Emulator,
    panes::{EmulatorPane, Pane, PaneDisplay},
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EMULATOR: Mutex<Emulator> = Mutex::new(Emulator::new());
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
    add_pane_to: Option<egui_tiles::TileId>, // Tile to add the new pane to
    pane_to_add: Option<Pane>,               // Pane variant to add
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        pane.title().into()
    }

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<Pane>, // Corrected generic type
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        true // Allow closing tabs
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane, // Corrected type
    ) -> egui_tiles::UiResponse {
        // Render the specific pane UI
        pane.render(ui);
        egui_tiles::UiResponse::None
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        tracing::trace!("Returning tile simplification options");
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

pub struct TemplateApp {
    tree: egui_tiles::Tree<Pane>,
    tree_behavior: TreeBehavior,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let span = tracing::info_span!("TemplateApp::default");
        let _guard = span.enter();

        tracing::info!("Creating new TemplateApp with comprehensive default layout");

        tracing::debug!("Initializing tile system");
        let mut tiles = egui_tiles::Tiles::default();

        // Create all panes we want to include
        tracing::debug!("Creating all panes for comprehensive layout");
        let memory_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Memory(
            crate::panes::emulator::memory::MemoryPane::default(),
        ))));
        let editor_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Editor(
            crate::panes::emulator::editor::EditorPane::default(),
        ))));
        let machine_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Machine(
            crate::panes::emulator::machine::MachinePane::default(),
        ))));
        let registers_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(
            EmulatorPane::Registers(crate::panes::emulator::registers::RegistersPane::default()),
        )));
        let controls_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(
            EmulatorPane::Controls(crate::panes::emulator::controls::ControlsPane),
        )));
        let output_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Output(
            crate::panes::emulator::io::IoPane::default(),
        ))));
        let cpu_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Cpu(
            crate::panes::emulator::cpu_state::CpuStatePane::default(),
        ))));
        let help_pane = tiles.insert_pane(Pane::EmulatorPanes(Box::new(EmulatorPane::Help(
            crate::panes::emulator::help::HelpPane::default(),
        ))));

        // Left main section with editor and memory
        let main_tabs = tiles.insert_tab_tile(vec![editor_pane, memory_pane]);

        // Right section with all other panes
        let right_top_tabs = tiles.insert_tab_tile(vec![registers_pane, machine_pane, cpu_pane]);
        let right_bottom_tabs = tiles.insert_tab_tile(vec![controls_pane, output_pane, help_pane]);

        // Create vertical split for right panes
        let right_split = tiles.insert_vertical_tile(vec![right_top_tabs, right_bottom_tabs]);

        // Create final horizontal layout
        let root = tiles.insert_horizontal_tile(vec![main_tabs, right_split]);

        // Set active tabs for initial focus on editor and memory
        if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
            tiles.get_mut(main_tabs)
        {
            tabs.set_active(editor_pane);
        }

        tracing::trace!("Creating tree with root ID: {}", root.0);
        let tree = egui_tiles::Tree::new("my_tree", root, tiles);

        tracing::info!("TemplateApp comprehensive initialization complete");
        Self {
            tree,
            tree_behavior: TreeBehavior::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let span = tracing::info_span!("TemplateApp::new");
        let _guard = span.enter();

        Default::default()
    }

    // Helper function to recursively build the pane menu
    fn add_pane_menu_items(
        &mut self,
        ui: &mut egui::Ui,
        pane_tree: &crate::panes::PaneTree,
        target_tile_id: egui_tiles::TileId,
    ) {
        match pane_tree {
            crate::panes::PaneTree::Pane(name, pane_variant) => {
                if ui.button(name).clicked() {
                    tracing::debug!(
                        "Queueing pane '{}' to add to tile {}",
                        name,
                        target_tile_id.0
                    );
                    // Queue the pane and the target tile ID for addition in the next frame
                    self.tree_behavior.add_pane_to = Some(target_tile_id);
                    self.tree_behavior.pane_to_add = Some(pane_variant.clone());
                    ui.close_menu();
                }
            }
            crate::panes::PaneTree::Children(name, children) => {
                ui.menu_button(name, |ui| {
                    for child in children {
                        self.add_pane_menu_items(ui, child, target_tile_id);
                    }
                });
            }
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let update_span = tracing::info_span!("TemplateApp::update");
        let _update_guard = update_span.enter();

        // Handle adding a pane if one was queued
        if let (Some(target_tile_id), Some(pane_to_add)) = (
            self.tree_behavior.add_pane_to.take(),
            self.tree_behavior.pane_to_add.take(),
        ) {
            // Insert the pane first to get its ID, avoiding simultaneous mutable borrows
            let new_pane_id = self.tree.tiles.insert_pane(pane_to_add);
            tracing::info!("Inserted new pane with ID {}", new_pane_id.0);

            // Now get the target tile mutably
            if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Tabs(tabs))) =
                self.tree.tiles.get_mut(target_tile_id)
            {
                tracing::info!(
                    "Adding pane {} to target tabs tile {}",
                    new_pane_id.0,
                    target_tile_id.0
                );
                tabs.add_child(new_pane_id);
                tabs.set_active(new_pane_id); // Make the new tab active
            } else {
                // Fallback: If the target tile wasn't a tab container
                tracing::warn!(
                    "Target tile {} is not a Tab container, trying to add to root.",
                    target_tile_id.0
                );
                if let Some(root_id) = self.tree.root() {
                    // Re-fetch the root tile mutably *after* inserting the pane
                    if let Some(egui_tiles::Tile::Container(container)) =
                        self.tree.tiles.get_mut(root_id)
                    {
                        container.add_child(new_pane_id);
                        if let egui_tiles::Container::Tabs(tabs) = container {
                            tabs.set_active(new_pane_id);
                        }
                        tracing::info!(
                            "Added pane {} to root container {}",
                            new_pane_id.0,
                            root_id.0
                        );
                    } else {
                        tracing::error!(
                            "Root tile {} is not a container, cannot add pane.",
                            root_id.0
                        );
                        // If the root isn't a container, we might need a different strategy,
                        // maybe replacing the root or creating a new structure.
                        // For now, we just remove the orphaned pane.
                        self.tree.tiles.remove(new_pane_id);
                        tracing::error!("Removed orphaned pane {}", new_pane_id.0);
                    }
                } else {
                    tracing::error!("No root tile found, cannot add pane.");
                    // Remove the orphaned pane if there's no root to add it to.
                    self.tree.tiles.remove(new_pane_id);
                    tracing::error!("Removed orphaned pane {}", new_pane_id.0);
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                // The top panel is often a good place for a menu bar:
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

                    // Panes Menu (dynamically generated from Pane::children)
                    let pane_menu_structure = Pane::children(); // Get the static structure

                    // Determine the target tile ID for adding new panes
                    // Try to use the first active *tabs* container, otherwise fallback to the root
                    let target_tile_id = self.tree.active_tiles().first().copied()
                        .or_else(|| self.tree.root()) // Fallback to root if no active tabs or root is not tabs
                        .unwrap_or_else(|| {
                            tracing::error!("No active tabs container or root tile found!");
                            // As a last resort, create a dummy TileId
                            egui_tiles::TileId::from_u64(u64::MAX)
                        });


                    match pane_menu_structure {
                        crate::panes::PaneTree::Children(root_label, children) => {
                            ui.menu_button(root_label, |ui| {
                                for child_tree in children {
                                    self.add_pane_menu_items(ui, &child_tree, target_tile_id);
                                }
                            });
                        }
                        crate::panes::PaneTree::Pane(_, _) => {
                            // Handle the case where the root is a single pane (less common for a menu)
                             tracing::warn!("Root of Pane::children() is a Pane, not Children. Menu might be limited.");
                             self.add_pane_menu_items(ui, &pane_menu_structure, target_tile_id);
                        }
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
                        let res = ui
                            .add(egui::Slider::new(&mut scale, 0.5..=5.0).text("UI Scale"));
                        if !res.dragged() && res.changed()
                        {
                            tracing::info!("Setting new UI scale: {}", scale);
                            ctx.set_zoom_factor(scale);
                        }
                        egui::widgets::global_theme_preference_buttons(ui);
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let tile_ui_span = tracing::info_span!("tile_tree_ui");
            let _tile_ui_guard = tile_ui_span.enter();

            self.tree.ui(&mut self.tree_behavior, ui);
            tracing::trace!("Tile tree UI render complete");
        });

        EMULATOR.lock().unwrap().update();

        ctx.request_repaint(); // update every frame
    }
}
