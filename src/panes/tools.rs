use base_converter::BaseConverter;
use serde::{Deserialize, Serialize};
use theme_editor::ThemeEditorPane;

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

    fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            ToolPanes::BaseConverter(converter) => converter.render(ui),
            ToolPanes::ThemeEditor(editor) => editor.render(ui),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children(
            "Tools".to_string(),
            vec![BaseConverter::children(), ThemeEditorPane::children()],
        )
    }
}
