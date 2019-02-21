// Copyright (c) 2019 Libgirl
//
// Released under Apache 2.0 license as described in the file LICENSE.txt.

/// the formulations of dynamic synapse should be refered to "Cerebral Cortex August 2004;14:933–944; doi:10.1093/cercor/bhh053"by Eugene M. Izhikevich et. al..
/// the name "markram1998" comes from the original paper, H. Markram et. al., Proc Natl Acad Sci USA 95:5323–5328(1998)

use crate::events::{Event, SpikeEvent};
use crate::models::{Neuron, NeuronActivity};
use crate::network::Network;
use crate::ode::rk4;
use crate::{Double, Parameters, Time};

#[derive(Debug)]
pub struct Model {
    pub a: Double,
    pub b: Double,
    pub c: Double,
    pub d: Double,
    pub v: Double,
    pub u: Double,
    v_th: Double,
    i_e: Double,
    spikes: Double,
    nid: i64,
    vesicle_pool_f: Double,
    vesicle_pool_d: Double,
    tau_vpf: Double,
    tau_vpd: Double,
    vpf0: Double,
}

impl Model {
    pub fn parameters() -> Parameters {
        let mut params = Parameters::new();
        params.insert("v_m".to_string(), -70.);
        params.insert("a".to_string(), 0.02);
        params.insert("b".to_string(), 0.2);
        params.insert("c".to_string(), -65.);
        params.insert("d".to_string(), 6.);
        params.insert("i_e".to_string(), 0.);
        params.insert("v_th".to_string(), 30.);
        params.insert("tau_vpf".to_string(), 1000.);
        params.insert("tau_vpd".to_string(), 800.);
        params.insert("vpf0".to_string(), 0.5.);
        params
    }

    fn get_or_default(params: &Parameters, key: &str) -> f64 {
        let default_params = Model::parameters();
        params
            .get(key)
            .cloned()
            .unwrap_or(*default_params.get(key).unwrap())
    }

    pub fn new(params: &Parameters) -> Model {
        let v_m = Model::get_or_default(params, "v_m");
        let a = Model::get_or_default(params, "a");
        let b = Model::get_or_default(params, "b");
        let c = Model::get_or_default(params, "c");
        let d = Model::get_or_default(params, "d");
        let i_e = Model::get_or_default(params, "i_e");
        let v_th = Model::get_or_default(params, "v_th");
        let tau_vpf = Model::get_or_default(params, "tau_vpf");
        let tau_vpd = Model::get_or_default(params, "tau_vpd");
        let vpf0 = Model::get_or_default(params, "vpf0");

        Model {
            a: a,
            b: b,
            c: c,
            d: d,
            v: v_m,
            u: b * v_m,
            v_th: v_th,
            i_e: i_e,
            spikes: 0.,
            nid: -1,
            tau_vpf,
            tau_vpd,
            vpf0,
        }
    }

    fn get_spike(&mut self, _lag: Double) -> Double {
        let spikes = self.spikes;
        self.spikes = 0.;
        spikes
    }

    fn add_spike(&mut self, e: &SpikeEvent) {
        self.spikes += e.weight() * e.multiplicity();
    }
}

impl Default for Model {
    fn default() -> Model {
        let mut params = Parameters::new();
        params.insert("v_m".to_string(), -70.);
        Model::new(&params)
    }
}

impl Neuron for Model {
    fn name(&self) -> String {
        String::from("Izhikevich_Markram1998")
    }

    fn set_params(&mut self, _params: &Parameters) {}

    fn update(&mut self, t: Double) -> NeuronActivity {
        let mut v = self.v;
        let mut u = self.u;
        let a = self.a;
        let b = self.b;
        let i_e = self.i_e;
        let i_syn = self.get_spike(t);
        let dt = Network::resolution();

        let d_v = move |y: Double| 0.04 * y * y + 5.0 * y + 140. - b * u + i_syn + i_e;
        v += rk4(d_v, v, dt);

        let d_u = move |y: Double| a * (v - y);
        u += rk4(d_u, u, dt);

        let mut activity = NeuronActivity::Silent;
        if v > self.v_th {
            let se = SpikeEvent::new();
            activity = NeuronActivity::Fires(se);
            v = self.c;
            u += self.d;
        }

        self.v = v;
        self.u = u;

        activity
    }

    fn handle_spike(&mut self, event: SpikeEvent) {
        self.add_spike(&event);
    }

    fn set_neuron_id(&mut self, nid: i64) {
        self.nid = nid;
    }

    fn neuron_id(&self) -> i64 {
        self.nid
    }

    fn new_spike_record(&mut self) {}

    fn set_spike_recording(&mut self, _is_on: bool) {}

    fn clear_spike_records(&mut self) {}

    fn get_spike_records(&self) -> Vec<Vec<Time>> {
        Vec::new()
    }

    fn get_property(&self, name: String) -> Double {
        match name.as_ref() {
            "v" => self.v,
            "v_th" => self.v_th,
            _ => 0.,
        }
    }

    fn set_property(&mut self, _name: String, _value: Double) {}
}
