use std::sync::atomic::Ordering;
use time;
use types::*;
use util::*;

pub struct TimeSettings {
    times_for: [f64; 2],
    inc_for: [f64; 2],
    moves_to_go: usize,
    ponder: bool,
    infinite: bool
}

impl TimeSettings {
    pub fn parse(mut self, params: &mut Params) -> Self {
        while let Some(option) = params.next() {
            match option {
                "wtime" => self.times_for[I_WHITE] = parse(params.next()),
                "btime" => self.times_for[I_BLACK] = parse(params.next()),
                "winc"  => self.inc_for[I_WHITE]   = parse(params.next()),
                "binc"  => self.inc_for[I_BLACK]   = parse(params.next()),
                "movestogo" => self.moves_to_go    = parse(params.next()),
                "ponder"   => self.ponder = true,
                "infinite" => self.infinite = true,
                _ => ()
            }
        }
        self
    }

    pub fn time(&self, side: usize) -> f64 {
        self.times_for[side] / 1000.0
    }

    pub fn inc(&self, side: usize) -> f64 {
        self.inc_for[side] / 1000.0
    }
}

impl Default for TimeSettings {
    fn default() -> Self {
        TimeSettings {
            times_for: [300000.0, 300000.0],
            inc_for: [0.0, 0.0],
            moves_to_go: 40,
            ponder: false,
            infinite: false
        }
    }
}

pub struct Timer {
    pub should_stop: Flag,
    settings: TimeSettings,
    nodes: Vec<usize>,
    times: Vec<f64>,
    side: usize,
    safety: f64,
    init: f64
}

impl Timer {
    pub fn new(flag: Flag, settings: TimeSettings) -> Self {
        Timer {
            should_stop: flag,
            settings: settings,
            nodes: vec![0],
            times: vec![0.0],
            side: !(I_WHITE | I_BLACK), // Initialize later
            safety: 0.1,
            init: 0.0
        }
    }

    pub fn default(flag: Flag) -> Self {
        Timer::new(flag, TimeSettings::default())
    }

    pub fn replace(&mut self, params: &mut Params) {
        let settings = TimeSettings::default().parse(params);
        *self = Timer::new(self.should_stop.clone(), settings);
    }

    pub fn start(&mut self, side: u8) {
        self.init = time::precise_time_s();
        self.side = side as usize;
        self.should_stop.store(false, Ordering::Relaxed);
    }

    pub fn toc(&mut self, node_count: usize) {
        self.nodes.push(node_count);
        let dt = self.elapsed();
        self.times.push(dt);
    }

    pub fn elapsed(&self) -> f64 {
        time::precise_time_s() - self.init
    }

    /// Return whether we should search to a given depth, or give the best move so far
    pub fn should_search(&self, depth: usize) -> bool {
        if depth <= 2 { return true }
        let estimate = self.times[depth-1] * self.nodes[depth-1] as f64 / self.nodes[depth-2] as f64;
        let alloc_time = (1.0 - self.safety) * self.settings.time(self.side) / self.settings.moves_to_go as f64
                         + self.settings.inc(self.side);

        !self.should_stop.load(Ordering::Relaxed) && (
        self.settings.infinite ||
        alloc_time - self.times[depth-1] > estimate * 0.3 ||
        alloc_time / 1.5 > self.elapsed())
    }
}
