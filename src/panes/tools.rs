use base_converter::BaseConverter;
use serde::{Deserialize, Serialize};
use theme_editor::ThemeEditorPane;

use crate::{emulator::Emulator, theme::ThemeSettings};

use super::{PaneDisplay, PaneTree};

mod base_converter;
mod theme_editor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolPanes {
    BaseConverter(BaseConverter),
    ThemeEditor(Box<ThemeEditorPane>),
}

impl PaneDisplay for ToolPanes {
    fn title(&self) -> impl Into<egui::WidgetText> {
        match self {
            ToolPanes::BaseConverter(_) => "Base Converter",
            ToolPanes::ThemeEditor(_) => "Theme Editor",
        }
    }

    fn render(&mut self, ui: &mut egui::Ui, emulator: &mut Emulator, theme: &mut ThemeSettings) {
        match self {
            ToolPanes::BaseConverter(converter) => converter.render(ui, emulator, theme),
            ToolPanes::ThemeEditor(editor) => editor.render(ui, emulator, theme),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Tools".to_string(),
            vec![BaseConverter::children(), ThemeEditorPane::children()],
        )
    }
}
