use crate::app::EMULATOR;
use crate::emulator::EmulatorCell;
use crate::panes::{Pane, PaneDisplay, PaneTree, RealPane};
use serde::{Deserialize, Serialize};

use super::EmulatorPane;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct RegistersPane {
    use_negitive: bool,
}

impl PaneDisplay for RegistersPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.checkbox(&mut self.use_negitive, "Show <0 as negative")
                .on_hover_text("Wether to display the 2s complement registers as being negative if bit 15 is set. EG FFFF vs -0001.");

            let mut lock = EMULATOR.lock();
            let emulator = &mut lock.as_deref_mut().unwrap();

            for i in 0..8 {
                ui.horizontal(|ui| {
                    ui.label(format!("R{}:", i));
                    register_view(ui, &mut emulator.r[i], self.use_negitive);
                });
            }

            ui.horizontal(|ui| {
                ui.label("PC:");
                register_view(ui, &mut emulator.pc, self.use_negitive);
            });

            ui.horizontal(|ui| {
                ui.label("MDR:");
                register_view(ui, &mut emulator.mdr, self.use_negitive);
            });

            ui.horizontal(|ui| {
                ui.label("MAR:");
                register_view(ui, &mut emulator.mar, self.use_negitive);
            });

            ui.horizontal(|ui| {
                ui.label("IR:");
                register_view(ui, &mut emulator.ir, self.use_negitive);
            });
        });
    }

    fn title(&self) -> String {
        "Registers".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Registers".to_string(),
            Pane::new(RealPane::EmulatorPanes(Box::new(EmulatorPane::Registers(
                Self::default(),
            )))),
        )
    }
}

fn register_view(ui: &mut egui::Ui, value_cell: &mut EmulatorCell, use_negative: bool) {
    if use_negative {
        let mut value: i16 = value_cell.get() as i16;
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(i16::MIN..=i16::MAX)
                .hexadecimal(4, false, true)
        )
        .on_hover_text("Drag to change value (hold Shift for slower). Click to edit value directly. Displayed as unsigned (0-65535)");

        if response.changed() {
            value_cell.set(value as u16);
        }
    } else {
        let mut value: u16 = value_cell.get();
        let response = ui.add(
            egui::DragValue::new(&mut value)
                .range(u16::MIN..=u16::MAX)
                .hexadecimal(4, false, true)
        )
        .on_hover_text("Drag to change value (hold Shift for slower). Click to edit value directly. Displayed as unsigned (0-65535)");

        if response.changed() {
            value_cell.set(value);
        }
    }
}
