pub mod emulator;
pub mod tools;

pub use emulator::EmulatorPane;
pub use tools::ToolPanes;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum PaneTree {
    Pane(String, Pane),
    Children(String, Vec<PaneTree>),
}

pub trait PaneDisplay {
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> impl Into<egui::WidgetText>;
    fn children() -> PaneTree;
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Pane {
    ToolPanes(Box<ToolPanes>),
    EmulatorPanes(Box<EmulatorPane>),
}

impl PaneDisplay for Pane {
    fn title(&self) -> impl Into<egui::WidgetText> {
        match self {
            Pane::ToolPanes(tools) => tools.title().into(),
            Pane::EmulatorPanes(emulator_panes) => emulator_panes.title().into(),
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            Pane::ToolPanes(pane) => pane.render(ui),
            Pane::EmulatorPanes(pane) => pane.render(ui),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Panes".to_string(),
            vec![ToolPanes::children(), EmulatorPane::children()],
        )
    }
}
