use bevy::prelude::Resource;

use crate::components::EnemyKind;

const WAVE_COUNT: usize = 20;
pub const TEST_WAVE_COUNT: u32 = 3;
pub const FULL_WAVE_COUNT: u32 = WAVE_COUNT as u32;

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

#[derive(Clone, Copy)]
pub struct EnemyGroup {
    pub kind: EnemyKind,
    pub count: u32,
    pub cooldown: f32,
    pub start_time: f32,
}

impl EnemyGroup {
    pub fn spawn_time(&self, spawned_count: u32) -> f32 {
        self.start_time + self.cooldown * spawned_count as f32
    }
}

pub struct Wave {
    pub groups: &'static [EnemyGroup],
}

impl Wave {
    pub fn enemy_count(&self) -> u32 {
        self.groups.iter().map(|group| group.count).sum()
    }
}

macro_rules! group {
    ($kind:ident, $count:expr, $cooldown:expr, $start_time:expr) => {
        EnemyGroup {
            kind: EnemyKind::$kind,
            count: $count,
            cooldown: $cooldown,
            start_time: $start_time,
        }
    };
}

pub const WAVES: [Wave; WAVE_COUNT] = [
    Wave {
        groups: &[group!(Grunt, 5, 2.5, 1.0)],
    },
    Wave {
        groups: &[group!(Grunt, 10, 1.5, 0.0), group!(Runner, 4, 1.2, 2.0)],
    },
    Wave {
        groups: &[group!(Runner, 6, 1.2, 0.0), group!(Grunt, 10, 0.8, 3.0)],
    },
    Wave {
        groups: &[
            group!(Grunt, 12, 0.7, 0.0),
            group!(Brute, 3, 2.2, 4.0),
            group!(Runner, 4, 1.0, 8.0),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 8, 0.9, 0.0),
            group!(Armored, 3, 2.0, 3.0),
            group!(Grunt, 10, 0.6, 7.0),
        ],
    },
    Wave {
        groups: &[
            group!(Brute, 5, 1.8, 0.0),
            group!(Grunt, 14, 0.55, 2.0),
            group!(Runner, 6, 0.9, 8.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 12, 0.7, 0.0),
            group!(Armored, 5, 1.6, 3.0),
            group!(Brute, 4, 1.8, 7.0),
        ],
    },
    Wave {
        groups: &[
            group!(Grunt, 18, 0.45, 0.0),
            group!(Brute, 6, 1.5, 2.0),
            group!(Runner, 8, 0.75, 8.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Armored, 8, 1.2, 0.0),
            group!(Runner, 12, 0.55, 3.0),
            group!(Brute, 5, 1.5, 8.0),
        ],
    },
    Wave {
        groups: &[
            group!(Brute, 8, 1.2, 0.0),
            group!(Armored, 8, 1.1, 2.0),
            group!(Grunt, 16, 0.4, 6.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 18, 0.45, 0.0),
            group!(Armored, 10, 0.95, 2.5),
            group!(Brute, 6, 1.3, 7.0),
        ],
    },
    Wave {
        groups: &[
            group!(Grunt, 20, 0.35, 0.0),
            group!(Brute, 10, 1.0, 2.0),
            group!(Armored, 8, 0.95, 6.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 20, 0.38, 0.0),
            group!(Brute, 8, 1.0, 3.0),
            group!(Armored, 12, 0.8, 5.0),
        ],
    },
    Wave {
        groups: &[
            group!(Armored, 14, 0.75, 0.0),
            group!(Brute, 12, 0.9, 2.0),
            group!(Runner, 18, 0.4, 7.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Brute, 14, 0.8, 0.0),
            group!(Grunt, 22, 0.3, 1.5),
            group!(Armored, 12, 0.75, 5.5),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 24, 0.32, 0.0),
            group!(Armored, 14, 0.7, 2.0),
            group!(Brute, 12, 0.85, 6.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Armored, 18, 0.62, 0.0),
            group!(Brute, 16, 0.75, 2.0),
            group!(Runner, 20, 0.35, 6.0),
        ],
    },
    Wave {
        groups: &[
            group!(Grunt, 24, 0.25, 0.0),
            group!(Runner, 24, 0.3, 1.0),
            group!(Armored, 16, 0.65, 4.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Brute, 20, 0.65, 0.0),
            group!(Armored, 18, 0.6, 1.5),
            group!(Runner, 24, 0.28, 5.0),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
    Wave {
        groups: &[
            group!(Runner, 28, 0.25, 0.0),
            group!(Brute, 22, 0.58, 1.0),
            group!(Armored, 22, 0.55, 2.5),
            group!(Titan, 1, 30.0, 18.0),
        ],
    },
];

pub fn wave(number: u32) -> Option<&'static Wave> {
    WAVES.get(number.checked_sub(1)? as usize)
}

pub fn enemies_in_wave(number: u32) -> u32 {
    wave(number).map_or(0, Wave::enemy_count)
}
