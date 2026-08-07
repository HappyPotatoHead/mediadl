// because there's a lot of repetition
pub trait Cycle {
    fn next(&mut self);
    fn prev(&mut self);
}

// this is for the ui
pub trait Named {
    fn name(&self) -> &'static str;
}

pub trait VerticalNavigation {
    fn move_up(&mut self);
    fn move_down(&mut self);
}

pub trait PanelNavigation {
    fn forward(&mut self);
    fn backward(&mut self);
}
