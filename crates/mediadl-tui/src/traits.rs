// for repetitive functions across multiple structs

//.Cycling through multiples values is more efficient this way than using if else
// especially for enum
// Applicable to other languages
// Much better than if else
pub trait Cycle {
    fn next(&mut self);
    fn prev(&mut self);
}

// this is for the ui
// returns the values in string form
// the &'static str only works when ur 100% confident that it will return
// a string
pub trait Named {
    fn name(&self) -> &'static str;
}

// self explainatory
pub trait VerticalNavigation {
    fn move_up(&mut self);
    fn move_down(&mut self);
}

pub trait PanelNavigation {
    fn forward(&mut self);
    fn backward(&mut self);
}
