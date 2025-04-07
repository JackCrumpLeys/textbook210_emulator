mod controls;
mod cpu_state;
mod editor;
mod help;
mod io;
mod machine;
mod memory;
mod registers;

use super::PaneDisplay;
use super::PaneTree;
use memory::MemoryPane;
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
    Memory(MemoryPane),
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
            EmulatorPane::Memory(pane) => pane.title(),
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
            EmulatorPane::Memory(pane) => pane.render(ui),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Emulator".to_owned(),
            vec![
                MemoryPane::children(),
                RegistersPane::children(),
                MachinePane::children(),
                EditorPane::children(),
                CpuStatePane::children(),
                IoPane::children(),
                HelpPane::children(),
                ControlsPane::children(),
            ],
        )
    }
}
