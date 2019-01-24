// Copyright (c) 2019 Libgirl
//
// Released under Apache 2.0 license as described in the file LICENSE.txt.

use connection_supervisor::ConnectionSupervisor;
use connections::{Connection, ConnectionInfo};
use connectors::Connector;
use events::{Event, SpikeEvent};
use models::cb_ath_lif;
use models::hodgkin_huxley;
use models::iaf;
use models::izhikevich;
use models::static_poisson;
use models::Neuron;
use models::NeuronActivity;
use models::NeuronType;
use populations::Population;
use {Double, Index, Num, Parameters, Time};

pub struct Network {
    neurons: Vec<Box<Neuron>>,
    populations: Vec<Box<Population>>,
    connection_supervisor: ConnectionSupervisor,
    next_neuron_id: Num,
    next_population_id: usize,
    resolution: f64,
    recording_neuron_ids: Vec<Num>,
}

impl Network {
    pub fn new() -> Network {
        Network {
            neurons: Vec::new(),
            populations: Vec::new(),
            connection_supervisor: ConnectionSupervisor::new(),
            next_neuron_id: 0,
            next_population_id: 0,
            resolution: Network::resolution(),
            recording_neuron_ids: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.neurons.clear();
        self.populations.clear();
        self.connection_supervisor.clear();
        self.recording_neuron_ids.clear();
        self.next_neuron_id = 0;
        self.next_population_id = 0;
        self.resolution = Network::resolution();
    }

    pub fn build_neuron(ntype: NeuronType, params: &Parameters) -> Box<Neuron> {
        match ntype {
            NeuronType::HodgkinHuxley => Box::new(hodgkin_huxley::Model::default()),
            NeuronType::IAF => Box::new(iaf::Model::default()),
            NeuronType::Izhikevich => Box::new(izhikevich::Model::new(params)),
            NeuronType::StaticPoisson => Box::new(static_poisson::Model::new(params)),
            NeuronType::ConductionBasedAdaptiveThresholdLIF => {
                Box::new(cb_ath_lif::Model::new(params))
            }
        }
    }

    pub fn create(
        &mut self,
        size: usize,
        ntype: NeuronType,
        params: &Parameters,
    ) -> Result<Population, &'static str> {
        if size == 0 {
            Err("invalid size")
        } else {
            let mut ids: Vec<Index> = Vec::new();
            for _ in 0..size {
                let neuron = Network::build_neuron(ntype, &params);
                let id = self.add_neuron(neuron);

                self.neurons[id].set_neuron_id(id as i64);
                ids.push(id as i64);
            }
            let population_id = self.next_population_id;
            self.next_population_id = self.next_population_id + 1;

            let population = Population::new(population_id, &ids);
            let population_box = Box::new(population.clone());
            self.populations.push(population_box);
            Ok(population)
        }
    }

    pub fn set_neuron_params(&mut self, id: Num, params: &Parameters) {
        self.neurons[id].set_params(params);
    }

    pub fn get_population_by_id(&self, id: usize) -> Box<Population> {
        self.populations[id].clone()
    }

    pub fn add_neuron(&mut self, neuron: Box<Neuron>) -> Num {
        let neuron_id = self.next_neuron_id;
        self.next_neuron_id = neuron_id + 1;

        self.neurons.push(neuron);
        neuron_id
    }

    pub fn connect<U: Connector, T: Connection>(
        &mut self,
        pre: &Population,
        post: &Population,
        conn: &U,
        syn: &T,
    ) -> Vec<Num> {
        conn.connect(pre, post, syn, &mut self.connection_supervisor)
    }

    fn evolve(&mut self, step: Double) {
        for i in 0..self.neurons.len() {
            if let NeuronActivity::Fires(_) = self.neurons[i].update(step) {
                let sender_id = self.neurons[i].neuron_id();
                self.connection_supervisor.propagate(sender_id, step);
                self.deliver_spike_event(sender_id);
            }
        }
    }

    pub fn run(&mut self, t: Time) {
        let steps: Double = t / self.resolution;
        let mut step = 0.0;
        for i in 0..self.recording_neuron_ids.len() {
            self.neurons[self.recording_neuron_ids[i]].new_spike_record();
        }
        while step < steps {
            self.evolve(step);
            step += self.resolution;
        }
    }

    pub fn resolution() -> Double {
        0.1
    }

    pub fn get_conn_info_by_id(&self, conn_id: Num) -> ConnectionInfo {
        self.connection_supervisor.get_conn_info_by_id(conn_id)
    }

    fn find_target_conn_infos(&self, source_id: Index) -> Vec<ConnectionInfo> {
        let conn_ids = self.connection_supervisor.get_connections(source_id);
        let mut v: Vec<ConnectionInfo> = Vec::new();
        for i in conn_ids {
            let conn = self.get_conn_info_by_id(i);
            v.push(conn);
        }
        v
    }

    fn deliver_spike_event(&mut self, sender_id: i64) {
        let t_conns = self.find_target_conn_infos(sender_id);
        for t in t_conns {
            let target_id = t.target as Num;
            let receiver = &mut self.neurons[target_id];
            let mut event = SpikeEvent::new();
            event.set_weight(t.weight);
            receiver.handle_spike(event);
        }
    }

    pub fn record_spikes(&mut self, population_id: usize) -> Result<(), String> {
        let population = self.get_population_by_id(population_id);
        for i in population.iter() {
            self.neurons[i as usize].set_spike_recording(true);
            self.recording_neuron_ids.push(i as usize); // notice in the future there might be cases that population shares some neurons
        }
        Ok(())
    }

    pub fn clear_spike_records(&mut self, population_id: usize) -> Result<(), String> {
        let population = self.get_population_by_id(population_id);
        for i in population.iter() {
            self.neurons[i as usize].clear_spike_records();
        }
        Ok(())
    }
    
    pub fn get_spike_records(&self) -> Vec<(Num, Vec<Vec<Time>>)> {
        let mut spike_records = Vec::new();
        for i in 0..self.recording_neuron_ids.len() {
            let neuron_id = self.recording_neuron_ids[i];
            let neuron = &self.neurons[neuron_id];
            let spike_histories = neuron.get_spike_records();
            spike_records.push((neuron_id, spike_histories));
        }
        spike_records
    }

}

impl Default for Network {
    fn default() -> Network {
        Network::new()
    }
}
