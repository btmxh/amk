#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GameLoopKind {
    Update = 0,
    Render = 1,
    Audio = 2,
}

impl GameLoopKind {
    pub fn index(&self) -> usize {
        *self as _
    }
}

pub(crate) const NUM_GAME_LOOPS: usize = 3;

pub trait GameLoop: Send {
    fn run(&self) {}
}

pub(crate) struct GameLoopContainer {
    pub data: [Option<Box<dyn GameLoop>>; NUM_GAME_LOOPS],
    pub relative_frequencies: [f64; NUM_GAME_LOOPS],
    pub timer: [f64; NUM_GAME_LOOPS],
}

impl GameLoopContainer {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            relative_frequencies: Default::default(),
            timer: Default::default(),
        }
    }

    pub fn insert(
        &mut self,
        kind: GameLoopKind,
        game_loop: Box<dyn GameLoop>,
        relative_frequency: f64,
    ) {
        let kind_index = kind.index();
        debug_assert!(self.data[kind_index].is_none());
        self.data[kind_index] = Some(game_loop);
        self.relative_frequencies[kind_index] = relative_frequency;
    }

    pub fn set_relative_frequency(&mut self, kind: GameLoopKind, relative_frequency: f64) {
        self.relative_frequencies[kind.index()] = relative_frequency;
    }

    pub fn get(&mut self, kind: GameLoopKind) -> Option<Box<dyn GameLoop>> {
        self.data[kind.index()].take()
    }

    pub fn empty(&self) -> bool {
        self.data.iter().all(|gl| gl.is_none())
    }

    pub fn run(&mut self) {
        for index in 0..NUM_GAME_LOOPS {
            if let Some(gl) = &self.data[index] {
                let mut timer_value = self.timer[index] + self.relative_frequencies[index];
                while timer_value >= 0.0 {
                    timer_value -= 1.0;
                    gl.run();
                }
                self.timer[index] = timer_value;
            }
        }
    }
}
