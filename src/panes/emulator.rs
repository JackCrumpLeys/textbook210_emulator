mod controls;
mod cpu_state;
mod editor;
mod help;
mod io;
mod machine;
mod registers;

use super::Pane;
use super::PaneDisplay;
use super::PaneTree;
use eframe::glow::INCR;
use serde::{Deserialize, Serialize};

pub use controls::ControlsPane;
pub use cpu_state::CpuStatePane;
pub use editor::EditorPane;
pub use help::HelpPane;
pub use io::IoPane;
pub use machine::MachinePane;
pub use registers::RegistersPane;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmulatorPane {
    Editor(EditorPane),
    Machine(MachinePane),
    Registers(RegistersPane),
    Output(IoPane),
    Help(HelpPane),
    Controls(ControlsPane),
    Cpu(CpuStatePane),
}

impl PaneDisplay for EmulatorPane {
    fn title(&self) -> impl Into<egui::WidgetText> {
        match self {
            EmulatorPane::Editor(pane) => pane.title(),
            EmulatorPane::Machine(pane) => pane.title(),
            EmulatorPane::Registers(pane) => pane.title(),
            EmulatorPane::Output(pane) => pane.title(),
            EmulatorPane::Help(pane) => pane.title(),
            EmulatorPane::Controls(pane) => pane.title(),
            EmulatorPane::Cpu(pane) => pane.title(),
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            EmulatorPane::Editor(pane) => pane.render(ui),
            EmulatorPane::Machine(pane) => pane.render(ui),
            EmulatorPane::Registers(pane) => pane.render(ui),
            EmulatorPane::Output(pane) => pane.render(ui),
            EmulatorPane::Help(pane) => pane.render(ui),
            EmulatorPane::Controls(pane) => pane.render(ui),
            EmulatorPane::Cpu(pane) => pane.render(ui),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Emulator".to_owned(),
            vec![
                PaneTree::Pane(
                    "Editor".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Editor(EditorPane::default()))),
                ),
                PaneTree::Pane(
                    "Machine Code".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Machine(MachinePane::default()))),
                ),
                PaneTree::Pane(
                    "Registers".to_string(),
                    Pane::EmulatorPanes(Box::new(
                        EmulatorPane::Registers(RegistersPane::default()),
                    )),
                ),
                PaneTree::Pane(
                    "Input/Output".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Output(IoPane::default()))),
                ),
                PaneTree::Pane(
                    "Controls".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Controls(ControlsPane::default()))),
                ),
                PaneTree::Pane(
                    "Debug".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Cpu(CpuStatePane::default()))),
                ),
                PaneTree::Pane(
                    "Help".to_string(),
                    Pane::EmulatorPanes(Box::new(EmulatorPane::Help(HelpPane::default()))),
                ),
            ],
        )
    }
}
