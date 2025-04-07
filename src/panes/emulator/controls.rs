use crate::app::EMULATOR;
use crate::emulator::parse::ParseOutput;
use crate::emulator::{CpuState, Emulator};
use crate::panes::emulator::editor::COMPILATION_ARTIFACTS;
use crate::panes::emulator::machine::BREAKPOINTS;
use crate::panes::{Pane, PaneDisplay, PaneTree};
use serde::{Deserialize, Serialize};

pub const MAX_OS_STEPS: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ControlsPane {
    speed: u32,
    ticks_between_updates: u32,
    #[serde(skip)]
    tick: u64,
    skip_os_emulation: bool,
}

impl Default for ControlsPane {
    fn default() -> Self {
        Self {
            speed: 1,
            ticks_between_updates: 2,
            tick: 0,
            skip_os_emulation: false,
        }
    }
}

impl PaneDisplay for ControlsPane {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.tick = self.tick.wrapping_add(1);
            let mut emulator = EMULATOR.lock().unwrap();
            let artifacts = COMPILATION_ARTIFACTS.lock().unwrap();
            let breakpoints = BREAKPOINTS.lock().unwrap();

            ui.group(|ui| {
                ui.label("Execution Speed");
                ui.horizontal(|ui| {
                    ui.label("Clocks per update:");
                    ui.add(egui::Slider::new(&mut self.speed, 1..=1000).logarithmic(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Update frequency:");
                    ui.add(
                        egui::Slider::new(&mut self.ticks_between_updates, 1..=100)
                            .text("ticks between updates")
                            .logarithmic(true),
                    );
                });
                ui.label("Higher speed values execute more instructions per update cycle.");

                // Add a skip OS emulation checkbox
                ui.checkbox(&mut self.skip_os_emulation, "Skip OS");
            });

            let mut os_steps = 0;
            ui.horizontal(|ui| {
                if ui.button("Small Step").clicked() {
                    emulator.micro_step();
                    if self.skip_os_emulation {
                        let old_running = emulator.running;
                        emulator.running = true;

                        while emulator.pc.get() < 0x3000
                            && os_steps < MAX_OS_STEPS
                            && emulator.running
                        {
                            emulator.step();
                            os_steps += 1;
                        }

                        if old_running && !emulator.running {
                            // make sure we can stop if needed
                            emulator.running = false;
                        } else {
                            emulator.running = old_running; // In every other case, restore the previous state
                        }
                    }
                }
                if ui.button("Step").clicked() {
                    emulator.step();
                    if self.skip_os_emulation {
                        let old_running = emulator.running;
                        emulator.running = true;

                        while emulator.pc.get() < 0x3000
                            && os_steps < MAX_OS_STEPS
                            && emulator.running
                        {
                            emulator.step();
                            os_steps += 1;
                        }

                        if old_running && !emulator.running {
                            // make sure we can stop if needed
                            emulator.running = false;
                        } else {
                            emulator.running = old_running; // In every other case, restore the previous state
                        }
                    }
                }

                if emulator.running {
                    if ui.button("Pause").clicked() {
                        emulator.running = false;
                    }
                } else if ui.button("Run").clicked() {
                    emulator.running = true;
                }

                if self.skip_os_emulation {
                    while emulator.pc.get() < 0x3000 && os_steps < MAX_OS_STEPS && emulator.running
                    {
                        emulator.step();
                        os_steps += 1;
                    }
                }

                if emulator.running {
                    // Automatic stepping logic when running
                    if self.tick % self.ticks_between_updates as u64 == 0 {
                        let mut i = 0;
                        while emulator.running && i < self.speed {
                            emulator.micro_step();
                            i += 1;

                            // Skip OS code if enabled during running mode
                            // Limit OS skipping to avoid freezing

                            while self.skip_os_emulation
                                && emulator.pc.get() < 0x3000
                                && os_steps < MAX_OS_STEPS
                                && emulator.running
                            {
                                emulator.step();
                                os_steps += 1;
                            }

                            if !emulator.pc.get() < 0x3000 {
                                // Check for breakpoints
                                let current_pc = emulator.pc.get() as usize;

                                if breakpoints.contains(&current_pc)
                                    && matches!(emulator.cpu_state, CpuState::Fetch)
                                // Break *before* fetching the instruction at the breakpoint
                                {
                                    emulator.running = false;
                                    log::info!("Breakpoint hit at address 0x{:04X}", current_pc);
                                    break;
                                }
                            }

                            if i >= self.speed {
                                break;
                            }
                        }
                    }
                }
            });

            // Reset button (distinct from Reset & Compile)
            if ui.button("Reset Emulator State").clicked() {
                *emulator = Emulator::new();

                // Optionally re-flash memory if needed, or clear it
                if !artifacts.last_compiled_source.is_empty() && artifacts.error.is_none() {
                    match Emulator::parse_program(&artifacts.last_compiled_source) {
                        Ok(ParseOutput {
                            machine_code,
                            orig_address,
                            ..
                        }) => {
                            emulator.flash_memory(machine_code, orig_address);
                        }
                        Err(_) => {
                            // Should not happen if artifacts are valid, but handle defensively
                            *emulator = Emulator::new();
                        }
                    }
                }
            }
        });
    }

    fn title(&self) -> String {
        "Controls".to_string()
    }

    fn children() -> PaneTree {
        PaneTree::Pane(
            "Controls".to_string(),
            Pane::EmulatorPanes(Box::new(super::EmulatorPane::Controls(
                ControlsPane::default(),
            ))),
        )
    }
}
