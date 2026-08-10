use crate::traits::{Cycle, VerticalNavigation};

#[derive(Debug, Default)]
pub struct OptionState {
    mode: OptionSelections,
}

#[derive(Debug, Default, PartialEq)]
pub enum OptionSelections {
    #[default]
    Configuration,
    Colour,
    Layout,
}

// Colour Selection and layout selection in the future

impl OptionState {
    pub fn get_mode(&self) -> &OptionSelections {
        &self.mode
    }
}

impl VerticalNavigation for OptionState {
    fn move_up(&mut self) {
        self.mode.prev();
    }
    fn move_down(&mut self) {
        self.mode.next();
    }
}

impl Cycle for OptionSelections {
    fn next(&mut self) {
        *self = match self {
            Self::Configuration => Self::Colour,
            Self::Colour => Self::Layout,
            Self::Layout => Self::Configuration,
        }
    }
    fn prev(&mut self) {
        *self = match self {
            Self::Configuration => Self::Layout,
            Self::Colour => Self::Configuration,
            Self::Layout => Self::Colour,
        }
    }
}

// impl Named for OptionSelections {
//     fn name(&self) -> &'static str {
//         match self {
//             Self::Colour => "Colour",
//             Self::Configuration => "Configuration",
//             Self::Layout => "Layout",
//         }
//     }
// }
