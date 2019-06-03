use std::sync::{Arc, Mutex};
use crate::operation::{RunMode, PassiveAgent, Configurable};
use crate::operation::op_population::PassivePopulation;
use crate::populations::HoldAgents;

pub struct SimplePassivePopulation<T>
where T: 'static + PassiveAgent + Send
{
    mode: RunMode,
    agents: Vec<Arc<Mutex<T>>>,
}

impl<T> Configurable for SimplePassivePopulation<T>
where T: 'static + PassiveAgent + Send
{
    fn config_mode(&mut self, mode: RunMode) {
        self.mode = mode;
        for agent in &self.agents {
            agent.lock().unwrap().config_mode(mode);
        }
    }

    fn config_channels(&mut self) {
        for agent in &self.agents {
            agent.lock().unwrap().config_channels();
        }
    }

    fn mode(&self) -> RunMode {
        self.mode
    }
}

impl<T> PassivePopulation for SimplePassivePopulation<T>
where T: 'static + PassiveAgent + Send
{}

impl<T> HoldAgents<T> for SimplePassivePopulation<T>
where T: 'static + PassiveAgent + Send
{
    fn agent_by_id(&self, n: usize) -> Arc<Mutex<T>> {
        Arc::clone(&self.agents[n])
    }    
}

impl<T>  SimplePassivePopulation<T>
where T: 'static + PassiveAgent + Send
{
    pub fn new() -> Arc<Mutex<SimplePassivePopulation<T>>> {
        Arc::new(Mutex::new(SimplePassivePopulation{
            mode: RunMode::Idle,
            agents: Vec::new(),
        }))
    }

    pub fn add(&mut self, agent: Arc<Mutex<T>>) {
        self.agents.push(agent);
    }
}
