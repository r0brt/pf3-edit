#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapsMode {
    Off,
    On,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditProfile {
    pub caps_mode: CapsMode,
    pub number_mode: bool,
    pub cols_mode: bool,
    pub bounds: Option<(usize, usize)>,
    pub tabs: Vec<usize>,
}

impl Default for EditProfile {
    fn default() -> Self {
        Self {
            caps_mode: CapsMode::Off,
            number_mode: false,
            cols_mode: false,
            bounds: None,
            tabs: vec![4, 8, 12, 16],
        }
    }
}
