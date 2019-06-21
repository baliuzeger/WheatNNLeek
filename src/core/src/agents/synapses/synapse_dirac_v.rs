use crossbeam_channel::Receiver as CCReceiver;
use crossbeam_channel::Sender as CCSender;
use std::sync::{Mutex, Arc};
use crate::{Ratio, Time, AcMx};
use crate::utils::Dimensionless;
use crate::operation::{
    Configurable, RunMode, Broadcast,  PassiveAgent, Passive, OpeChs, PassiveBackOpeChs, ActiveAgent,
};
use crate::operation::op_agent::{ConsecutivePassiveAgent};
use crate::signals::dirac_delta_voltage::{
    SynapseComponentDiracV, PreSynDiracV, PostSynDiracV,
    SmplChsCarPreSynDiracV, PostSynChsCarDiracV,
};
use crate::agents::{Agent};
use crate::agents::synapses::{SynapseFlag};
use crate::connectivity::{
    Generator, Acceptor,
    AppendableForeEnd, AppendableTwoWayBackEnd,
    linker::Linker,
};


pub struct SynapseModel
{
    w: Ratio, // weight
    w_max: Ratio, // upper bound of weight
    w_min: Ratio, // lower bound of weight
    delay: Time, // delay time
    stdp_pre_amount: Ratio, // max amount of STDP on pre spike.
    tau_stdp_pre: Time, // stdp decay time constand on pre spike.
    stdp_post_amount: Ratio,  // max amount of STDP on post spike.
    tau_stdp_post: Time, // stdp decay time constand on post spike.
    pre_firing_history: Vec<Time>,
    post_firing_history: Vec<Time>,
    ope_chs_gen: OpeChs<()>,
    component: SynapseComponentDiracV,
}

impl Configurable for SynapseModel
{
    fn config_mode(&mut self, mode: RunMode) {
        self.component.config_mode(mode);
    }
    
    fn config_channels(&mut self) {
        self.component.config_channels();
    }

    fn mode(&self) -> RunMode {
        self.component.mode()
    }
}

impl Agent for SynapseModel {}

impl PassiveAgent for SynapseModel
{    
    fn recheck_mode(&mut self) {
        // println!("SynapseS0S1 recheck_mode().");
        self.component.recheck_mode();
    }

    fn report_sender(&self) -> CCSender<()> {
        self.ope_chs_gen.report_sender()
    }

    fn passive_back_ope_chs(&self) -> PassiveBackOpeChs {
        self.ope_chs_gen.passive_back_ope_chs()
    }
}

impl Passive for SynapseModel {
    fn run(&mut self) {
        <Self as ConsecutivePassiveAgent>::run(self);
    }

    fn confirm_sender(&self) -> CCSender<Broadcast> {
        self.ope_chs_gen.confirm_sender()
    }
    
    fn confirm_receiver(&self) -> CCReceiver<Broadcast> {
        self.ope_chs_gen.confirm_receiver()
    }
}

impl ConsecutivePassiveAgent for SynapseModel {
    fn respond(&mut self) {

        let v_s: Vec<PreSynDiracV> = self.component.ffw_accepted().collect();
        for s in v_s {
            let acting_time = s.t + self.delay;
            self.pre_firing_history.push(acting_time);
            self.stdp_on_pre(acting_time);
            self.component.feedforward(PostSynDiracV {
                v: s.v,
                t: acting_time,
                w: self.w,
            })
        };

        match self.component.flag() {
            SynapseFlag::Static => (),
            SynapseFlag::STDP => {
                let v_t: Vec<Time> = self.component.fbw_accepted().map(|s| s.0).collect();
                for t in v_t {
                    self.post_firing_history.push(t);
                    self.stdp_on_post(t);
                }
            }
        }
    }

    fn passive_sync_chs_sets(&self) -> Vec<PassiveBackOpeChs> {
        self.component.passive_sync_chs_sets()
    }
}

// impl AcceptorDiracV for SynapseModel {}

impl Acceptor<SmplChsCarPreSynDiracV> for SynapseModel {}

// impl SynapseGeneratorDiracV for SynapseModel {}

impl Generator<PostSynChsCarDiracV> for SynapseModel {}


impl SynapseModel {
    pub fn config_syn_flag(&mut self, flag: SynapseFlag) {
        self.component.config_syn_flag(flag);
    }

    fn stdp_on_pre(&mut self, pre_t: Time) {
        let post_len = self.post_firing_history.len();
        if post_len > 0 {
            let w_new = self.w + self.stdp_pre_amount
                * (
                    (self.post_firing_history[post_len - 1] - pre_t) / self.tau_stdp_pre
                ).exp();
            self.update_w(w_new);
            
        }
    }

    fn stdp_on_post(&mut self, post_t: Time) {
        let pre_len = self.pre_firing_history.len();
        if pre_len > 0 {
            let w_new = self.w + self.stdp_post_amount
                * (
                    (self.pre_firing_history[pre_len - 1] - post_t) / self.tau_stdp_post
                ).exp();
            self.update_w(w_new);
        }
    }

    fn update_w(&mut self, w_new: Ratio) {
        if w_new >= self.w_max {
            self.w = self.w_max;
        } else if w_new <= self.w_min {
            self.w = self.w_min
        } else {
            self.w = w_new;
        }        
    }
    
}

struct ParamsSynapseDiracV {
    pub w: Ratio, // weight
    pub w_max: Ratio, // upper bound of weight
    pub w_min: Ratio, // lower bound of weight
    pub delay: Time, // delay time
    pub stdp_pre_amount: Ratio, // max amount of STDP on pre spike.
    pub tau_stdp_pre: Time, // stdp decay time constand on pre spike.
    pub stdp_post_amount: Ratio,  // max amount of STDP on post spike.
    pub tau_stdp_post: Time, // stdp decay time constand on post spike.
}

impl ParamsSynapseDiracV {
    pub fn build_to_active<G, AA>(&self, pre: AcMx<G>, post: AcMx<AA>) -> AcMx<SynapseModel>
    where G: 'static + AppendableForeEnd<SmplChsCarPreSynDiracV> + Send,
          AA: 'static + ActiveAgent + AppendableTwoWayBackEnd<PostSynChsCarDiracV> + Send,
    {
        let pre_linker = Linker::new();
        let post_linker = Linker::new();
        let syn = Arc::new(Mutex::new(SynapseModel {
            w: self.w,
            w_max: self.w_max,
            w_min: self.w_min,
            delay: self.delay,
            stdp_pre_amount: self.stdp_pre_amount,
            tau_stdp_pre: self.tau_stdp_pre,
            stdp_post_amount: self.stdp_post_amount,
            tau_stdp_post: self.tau_stdp_post,
            pre_firing_history: Vec::new(),
            post_firing_history:  Vec::new(),
            ope_chs_gen: OpeChs::new(),
            component: SynapseComponentDiracV::new_on_active(
                pre.clone(),
                pre_linker.clone(),
                post.clone(),
                post_linker.clone()
            ),
        }));
        pre.lock().unwrap().add_passive(syn.clone(), pre_linker);
        post.lock().unwrap().add_passive(syn.clone(), post_linker);
        syn
    }

    pub fn build_to_passive<G, AA>(&self, pre: AcMx<G>, post: AcMx<AA>) -> AcMx<SynapseModel>
    where G: 'static + AppendableForeEnd<SmplChsCarPreSynDiracV> + Send,
          AA: 'static + PassiveAgent + AppendableTwoWayBackEnd<PostSynChsCarDiracV> + Send,
    {
        let pre_linker = Linker::new();
        let post_linker = Linker::new();
        let syn = Arc::new(Mutex::new(SynapseModel {
            w: self.w,
            w_max: self.w_max,
            w_min: self.w_min,
            delay: self.delay,
            stdp_pre_amount: self.stdp_pre_amount,
            tau_stdp_pre: self.tau_stdp_pre,
            stdp_post_amount: self.stdp_post_amount,
            tau_stdp_post: self.tau_stdp_post,
            pre_firing_history: Vec::new(),
            post_firing_history:  Vec::new(),
            ope_chs_gen: OpeChs::new(),
            component: SynapseComponentDiracV::new_on_passive(
                pre.clone(),
                pre_linker.clone(),
                post.clone(),
                post_linker.clone()
            ),
        }));
        pre.lock().unwrap().add_passive(syn.clone(), pre_linker);
        post.lock().unwrap().add_passive(syn.clone(), post_linker);
        syn
    }
    
}
