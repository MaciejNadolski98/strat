use bevy::prelude::Resource;

pub const TEST_WAVE_COUNT: u32 = 3;
pub const FULL_WAVE_COUNT: u32 = 20;

#[derive(Default, Clone, Copy)]
pub enum RunModeKind {
    Test,
    #[default]
    Full,
}

#[derive(Default, Resource, Clone, Copy)]
pub struct RunMode {
    pub kind: RunModeKind,
}

impl RunMode {
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Self {
        let mut mode = Self::default();

        for arg in args {
            match arg.as_str() {
                "--test" => {
                    mode.kind = RunModeKind::Test;
                }
                _ => {}
            }
        }

        mode
    }

    pub fn final_wave(self) -> u32 {
        match self.kind {
            RunModeKind::Test => TEST_WAVE_COUNT,
            RunModeKind::Full => FULL_WAVE_COUNT,
        }
    }

    pub fn label(self) -> &'static str {
        match self.kind {
            RunModeKind::Test => "test",
            RunModeKind::Full => "full",
        }
    }
}
