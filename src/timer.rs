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
    pub fn new() -> TimeSettings {
        TimeSettings {
            times_for: [300000.0, 300000.0],
            inc_for: [0.0, 0.0],
            moves_to_go: 40,
            ponder: false,
            infinite: false
        }
    }

    pub fn fill(params: &mut Params) -> TimeSettings {
        let mut s = TimeSettings::new();

        while let Some(p) = params.next() {
            match p {
                "wtime" => s.times_for[I_WHITE] = parse_or_fail(params.next()),
                "btime" => s.times_for[I_BLACK] = parse_or_fail(params.next()),
                "winc"  => s.inc_for[I_WHITE]   = parse_or_fail(params.next()),
                "binc"  => s.inc_for[I_BLACK]   = parse_or_fail(params.next()),
                "movestogo" => s.moves_to_go    = parse_or_fail(params.next()),
                "ponder"   => s.ponder = true,
                "infinite" => s.infinite = true,
                _ => ()
            }
        }
        s
    }

    pub fn time(&self, side: usize) -> f64 {
        self.times_for[side] / 1000.0
    }

    pub fn inc(&self, side: usize) -> f64 {
        self.inc_for[side] / 1000.0
    }
}

pub struct Timer {
    settings: TimeSettings,
    nodes: Vec<usize>,
    times: Vec<f64>,
    side: usize,
    safety: f64,
    init: f64
}

impl Timer {
    pub fn new(params: &mut Params) -> Timer {
        Timer {
            settings: TimeSettings::fill(params),
            nodes: vec![0],
            times: vec![0.0],
            side: !(I_WHITE | I_BLACK), // Initialize later
            safety: 0.1,
            init: 0.0
        }
    }

    pub fn start(&mut self, side: u8) {
        self.init = time::precise_time_s();
        self.side = side as usize;
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

        alloc_time - self.times[depth-1] > estimate * 0.3
    }
}
