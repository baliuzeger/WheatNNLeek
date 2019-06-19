// Copyright (c) 2019 Libgirl
//
// Released under Apache 2.0 license as described in the file LICENSE.txt.
// multi-in/out S1Pre; for testing the parogramming framework.

use crossbeam_channel::Receiver as CCReceiver;
use crossbeam_channel::Sender as CCSender;
use std::sync::{Mutex, Arc};
use crate::{AcMx};
use crate::signals::s1::{
    NeuronPostSynComponentS1, MultiInComponentS1,
    NeuronAcceptorS1, 
    SimpleChsCarrierS1, SimpleLinkerS1, PostSynChsCarrierS1, PostSynLinkerS1,
    StdpBkwd0,
};
use crate::signals::s0::{
    MultiOutComponentS0,
    SimpleGeneratorS0,
    SimpleChsCarrierS0, SimpleLinkerS0,
    S0,
};
use crate::connectivity::{
    Generator, Acceptor,
    AppendableForeEnd, AppendableOneWayBackEnd, AppendableTwoWayBackEnd,
    ActiveAcceptor, PassiveAcceptor,
    ActiveGenerator, PassiveGenerator,
};
use crate::operation::{
    ActiveAgent, Configurable, Broadcast, Fired, RunMode,
    PassiveBackOpeChs, Active, OpeChs,
};
use crate::operation::op_agent::FiringActiveAgent;
use crate::agents::{Agent, Neuron};

use uom::si::f64::Time;

pub struct NeuronT {
    ope_chs_gen: OpeChs<Fired>,
    out_s0: MultiOutComponentS0,
    post_syn_s1: NeuronPostSynComponentS1,
    device_in_s1: MultiInComponentS1,
    gen_value: i32,
    proc_value: i32,
    event_cond: Option<i32>,
    stock: Vec<FwdEndProduct>,
}

struct FwdEndProduct {
    pub msg: i32,
    pub proc: i32,
}

impl NeuronAcceptorS1 for NeuronT {}
impl Neuron for NeuronT {}

impl Acceptor<SimpleChsCarrierS1> for NeuronT {}

impl AppendableOneWayBackEnd<SimpleChsCarrierS1> for NeuronT {
    fn add(&mut self, pre: AcMx<dyn Generator<SimpleChsCarrierS1> + Send>, linker: AcMx<SimpleLinkerS1>) {
        self.device_in_s1.add(pre, linker);
    }
}

impl Acceptor<PostSynChsCarrierS1> for NeuronT {}

impl AppendableTwoWayBackEnd<PostSynChsCarrierS1> for NeuronT {
    fn add_active(&mut self, pre: AcMx<dyn ActiveGenerator<PostSynChsCarrierS1> + Send>, linker: AcMx<PostSynLinkerS1>) {
        self.post_syn_s1.add_active(pre, linker);
    }

    fn add_passive(&mut self, pre: AcMx<dyn PassiveGenerator<PostSynChsCarrierS1> + Send>, linker: AcMx<PostSynLinkerS1>) {
        self.post_syn_s1.add_passive(pre, linker);
    }
}

impl SimpleGeneratorS0 for NeuronT {}

impl Generator<SimpleChsCarrierS0> for NeuronT {}

impl AppendableForeEnd<SimpleChsCarrierS0> for NeuronT {
    fn add_active(&mut self, post: AcMx<dyn ActiveAcceptor<SimpleChsCarrierS0> + Send>, linker: AcMx<SimpleLinkerS0>) {
        self.out_s0.add_active_target(post, linker);
    }

    fn add_passive(&mut self, post: AcMx<dyn PassiveAcceptor<SimpleChsCarrierS0> + Send>, linker: AcMx<SimpleLinkerS0>) {
        self.out_s0.add_passive_target(post, linker);
    }
}

impl Configurable for NeuronT {
    fn config_mode(&mut self, mode: RunMode) {
        self.post_syn_s1.config_mode(mode);
        self.device_in_s1.config_mode(mode);
        self.out_s0.config_mode(mode);
    }
    
    fn config_channels(&mut self) {
        self.post_syn_s1.config_channels();
        self.device_in_s1.config_channels();
        self.out_s0.config_channels();   
    }

    fn mode(&self) -> RunMode {
        match (self.out_s0.mode(), self.post_syn_s1.mode(), self.device_in_s1.mode()) {
            (out_s0, post_syn_s1, device_in_s1) if out_s0 == post_syn_s1 && out_s0 == device_in_s1 => out_s0,
            (out_s0, post_syn_s1, device_in_s1) => panic!(
                "components of NeuronT have different modes, out_s0: {:?}, post_syn_s1: {:?}, device_in_s1: {:?}.",
                out_s0, post_syn_s1, device_in_s1
            ),
        }
    }
}

impl Agent for NeuronT {}

impl ActiveAgent for NeuronT {}

impl Active for NeuronT {
    type Report = Fired;
    fn run(&mut self, dt: Time, time: Time) {
        <Self as FiringActiveAgent>::run(self, dt, time);
    }
    fn confirm_sender(&self) -> CCSender<Broadcast> {
        self.ope_chs_gen.confirm_sender()
    }
    
    fn confirm_receiver(&self) -> CCReceiver<Broadcast> {
        self.ope_chs_gen.confirm_receiver()
    }
    
    fn report_receiver(&self) -> CCReceiver<<Self as Active>::Report> {
        self.ope_chs_gen.report_receiver()
    }
    
    fn report_sender(&self) -> CCSender<<Self as Active>::Report> {
        self.ope_chs_gen.report_sender()
    }
}

impl FiringActiveAgent for NeuronT {
    fn end(&mut self) {
        self.accept();
    }
    
    fn evolve(&mut self, _dt: Time, _time: Time) -> Fired {
        self.proc_value += 1;
        self.gen_value += 1;
        self.accept();
        match self.event_cond {
            None => {
                // println!("agnet a go on. gen: {}, proc: {}.",  self.gen_value, self.proc_value);
                Fired::N   
            },
            Some(n) => {
                match self.proc_value % n {
                    0 => {
                        println!("agnet c fire. gen: {}, proc: {}.",  self.gen_value, self.proc_value);
                        self.generate();
                        Fired::Y
                    },
                    _ => {
                        // println!("agnet a go on. gen: {}, proc: {}.",  self.gen_value, self.proc_value);
                        Fired::N
                    },
                }
            }
        }
    }

    fn passive_sync_chs_sets(&mut self) -> Vec<PassiveBackOpeChs> {
        // change to iterator later.
        let mut v1 = self.out_s0.passive_sync_chs_sets();
        let mut v2 = self.post_syn_s1.passive_sync_chs_sets();
        v1.append(&mut v2);
        v1
    }
}

impl NeuronT {
    pub fn new(gen_value: i32, proc_value: i32, event_cond: Option<i32>) -> AcMx<NeuronT> {
        Arc::new(Mutex::new(
            NeuronT {
                ope_chs_gen: OpeChs::new(),
                out_s0: MultiOutComponentS0::new(),
                device_in_s1: MultiInComponentS1::new(),
                post_syn_s1: NeuronPostSynComponentS1::new(),
                gen_value,
                proc_value,
                event_cond,
                stock: Vec::new(),
            }
        ))
    }
    
    fn generate(&self) {
        self.out_s0.feedforward(S0 {
            msg_gen: self.gen_value,
        });
        self.post_syn_s1.feedbackward(StdpBkwd0 {
            msg: self.gen_value,
        })
    }

    fn accept(&mut self) {
        let mut acc = self.post_syn_s1.ffw_accepted().map(|s| FwdEndProduct {
            msg: s.msg_gen,
            proc: self.proc_value,
        }).chain(
            self.device_in_s1.ffw_accepted().map(|s| FwdEndProduct {
                msg: s.msg_gen,
                proc: self.proc_value,
            })  
        ).collect::<Vec<FwdEndProduct>>();

        // for demo accepting
        for msg in &acc {
            println!(
                "agent c accept: gen: {}, proc: {}. self-gen: {}.",
                msg.msg,
                msg.proc,
                self.gen_value,
            )
        }
        
        self.stock.append(&mut acc);
    }

    // pub fn print_values(&self) {
    //     println!("gen: {}, proc: {}.", self.gen_value, self.proc_value);
    // }
    
    pub fn show(&self) {
        for msg in &self.stock {
            println!(
                "agent c buffer: gen: {}, proc: {}.",
                msg.msg,
                msg.proc
            )
        }
    }
}
