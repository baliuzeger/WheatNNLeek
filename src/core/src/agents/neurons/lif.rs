// Copyright (c) 2019 Libgirl
//
// Released under Apache 2.0 license as described in the file LICENSE.txt.
// Integrate-and-fire model
use uom::si::f64::Time;
use uom::si::time::millisecond;
use uom::si::f64::ElectricalResistance as Resistance;
use uom::si::electrical_resistance::megaohm;
use uom::si::f64::ElectricCurrent as Current;
use uom::si::electric_current::nanoampere;
use uom::si::f64::ElectricPotential as Voltage;
use uom::si::electric_potential::millivolt;

use crate::agents::neurons::Neuron;
use crate::signals::simple_post_syn::SimpleNeuronAcceptor;

pub struct NeuronModel {
    v_rest: Voltage,   // Membrane resting potential
    r_m: Resistance,   // Membrane resistance
    tau_m: Time,       // Membrane time constant
    v: Voltage,        // Membrane Voltage
    v_th: Voltage,     // Thresold Voltage of firing
    i_e: Current,      // constant current injection
}

impl SimpleNeuronAcceptor for NeuronModel {}
impl Neuron for NeuronModel {}
