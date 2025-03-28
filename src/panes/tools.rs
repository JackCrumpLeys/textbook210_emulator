use base_converter::BaseConverter;
use serde::{Deserialize, Serialize};

use super::{Pane, PaneDisplay, PaneTree};

mod base_converter;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolPanes {
    BaseConverter(BaseConverter),
}

impl PaneDisplay for ToolPanes {
    fn title(&self) -> impl Into<egui::WidgetText> {
        match self {
            ToolPanes::BaseConverter(_) => "Base Converter",
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        match self {
            ToolPanes::BaseConverter(converter) => converter.render(ui),
        }
    }

    fn children() -> PaneTree {
        PaneTree::Children("Tools".to_string(), vec![BaseConverter::children()])
    }
}
