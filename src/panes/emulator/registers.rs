use crate::app::{base_to_base, EMULATOR};
use crate::emulator::EmulatorCell;
use crate::panes::{Pane, PaneDisplay, PaneTree};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RegistersPane {
    display_base: u32,
}

impl Default for RegistersPane {
    fn default() -> Self {
        Self { display_base: 16 }
    }
}

impl PaneDisplay for RegistersPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        let mut lock = EMULATOR.lock();
        let emulator = &mut lock.as_deref_mut().unwrap();

        for i in 0..8 {
            ui.horizontal(|ui| {
                ui.label(format!("R{}:", i));
                register_view(ui, &mut emulator.r[i], self.display_base);
            });
        }

        ui.horizontal(|ui| {
            ui.label("PC:");
            register_view(ui, &mut emulator.pc, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("MDR:");
            register_view(ui, &mut emulator.mdr, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("MAR:");
            register_view(ui, &mut emulator.mar, self.display_base);
        });

        ui.horizontal(|ui| {
            ui.label("IR:");
            register_view(ui, &mut emulator.ir, self.display_base);
        });
    }

    fn title(&self) -> String {
        "Registers".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Registers".to_string(),
            Pane::EmulatorPanes(Box::new(EmulatorPane::Registers(Self::default()))),
        )
    }
}

fn register_view(ui: &mut egui::Ui, value_cell: &mut EmulatorCell, base: u32) {
    let mut value = value_cell.get() as i16;
    ui.add(egui::DragValue::new(&mut value));
    value_cell.set(value as u16);
    ui.label(base_to_base(
        10,
        base,
        &(value_cell.get() as u32).to_string(),
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    ));
}
