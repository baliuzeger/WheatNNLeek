use crate::components::{NeuronPreSynComponent};
// use crate::components::{MultiInComponent, MultiOutComponent, SingleInComponent, SingleOutComponent};
use crate::connectivity::{Generator, PassiveAcceptor, ActiveAcceptor};
use crate::connectivity::simple_joint::SimpleChsCarrier;

#[derive(Copy, Clone)]
pub struct FwdPreS1 {
    pub msg_gen: i32
}

// pub struct BkwdPreS1 {
//     pub msg_gen: i32
// }

type DynAA = dyn ActiveAcceptor<SimpleChsCarrier<FwdPreS1>> + Send;
type DynPA = dyn PassiveAcceptor<SimpleChsCarrier<FwdPreS1>> + Send;
pub type NeuronPreSynComponentS1 = NeuronPreSynComponent<DynAA, DynPA, FwdPreS1>;
// pub type MultiOutComponentS1Pre = MultiOutComponent<dyn ActiveAcceptor<FwdPreS1>, dyn PassiveAcceptor<FwdPreS1>, FwdPreS1>;

// pub type MultiInComponentS1Pre = MultiInComponent<dyn Generator<FwdPreS1>, FwdPreS1>;

// SingleOut to PassiveSingleOut not implemented yet.
// pub type SingleOutComponentS1Pre = SingleOutComponent<dyn ActiveAcceptor<FwdPreS1>, dyn PassiveAcceptor<FwdPreS1>, FwdPreS1>;

// pub type SingleInComponentS1Pre = SingleInComponent<dyn Generator<FwdPreS1>, FwdPreS1>;
