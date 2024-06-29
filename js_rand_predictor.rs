use std::{collections::HashMap, task::Context};

use z3::{ast::BV, Config, Solver};

pub struct JsRandPredictor {
    fuel: Vec<f64>,
    cfg: Config,
    ctx: Context,
    se_state0: BV,
    se_state1: BV,

    solver: Solver,
}

impl JsRandPredictor {
    pub fn new() -> Self {
        let fuel = fuel_from_js();

        let cfg = Config::new();
        let ctx = Context::new(&cfg);
        let mut new = JsRandPredictor {
            fuel,
            cfg,
            ctx,
            se_state0: BV::new_const(ctx, "se_state0", 64),
            se_state1: BV::new_const(ctx, "se_state1", 64),

            solver: Solver::new(&ctx),
        };

        new.fuel();

        new
    }

    pub fn predict(&mut self) -> f64 {
        self.fuel();
        self.add_data();

        let mut model = self.solver.get_model().unwrap();
        let mut states = HashMap::new();

        for state in model.iter() {
            states.insert(state.name(), state);
        }

        info!("{:?}", states);

        let mut state0 = states.get("se_state0").unwrap();

        let ulong = (state0 >> 12) | 0x3FF0000000000000;

        info!("idk {}", ulong);
    }

    pub fn add_data(&mut self) {
        for i in 0..self.fuel.len() {
            let fuel = self.fuel[i];

            let mut se_s1 = self.se_state0;
            let mut se_s0 = self.se_state1;
            self.se_state0 = se_s0;

            se_s1 ^= se_s1 << 23;
            se_s1 ^= se_s1.bvlshr(17);
            se_s1 ^= se_s0;
            se_s1 ^= se_s0.bvlshr(26);
            self.se_state1 = se_s1;

            let mantissa = self.fuel[i] + 1 & ((1 << 52) - 1);
            self.solver.assert(mantissa = self.se_state0.bvlshr(12));
        }
    }

    pub fn fuel(&mut self) -> Vec<f64> {
        let mut fuel = Vec::new();
    
        for i in 0..5 {
            fuel.push(js_sys::Math::random());
        }
    
        // We have to flip it due to JS's
        // pop array, its Last in First Out, so we have to reverse it.
        fuel.into_iter().rev().collect()
    }
}

pub fn test() {
    let mut predictor = JsRandPredictor::new();

    predictor.predict();
}