#![allow(dead_code)]
use std::collections::{BTreeSet, HashMap, HashSet};

use gate::{GatePtr, IoDir};
use graph::{Graph, GraphPtr};
use js_types::{JsGateParams, IOmap, PortParams, TargetParams, JsMonitorParams, JsVec3vl, JsAlarmStruct};
use link::LinkTarget;
use operations::ReturnValue;
use vector3vl::Vec3vl;
use wasm_bindgen::prelude::*;

mod graph;
mod gate;
mod link;
mod js_types;
mod vector3vl;
mod operations;

mod cell_arith;
mod cell_bus;
mod cell_dff;
mod cell_fsm;
mod cell_gates;
mod cell_io;
mod cell_memory;
mod cell_mux;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(module = "/src/engines/wasm-js-functions.mjs")]
extern "C" {
    fn sendUpdates(tick: u32, pendingEvents: bool, updates: Vec<UpdateStruct>);
    fn triggerMemoryUpdate(graphId: String, gateId: String, address: i32, bits: u32, avec: Vec<u32>, bvec: Vec<u32>);
    fn triggerFSMCurrentStateChange(graphId: String, gateId: String, currentState: u32);
    fn triggerFSMNextTransChange(graphId: String, gateId: String, transitionId: Option<String>);
    fn postMonitorValue(monitorId: u32, tick: u32, bits: u32, avec: Vec<u32>, bvec: Vec<u32>, stopOnTrigger: Option<bool>, oneShot: Option<bool>);
    fn updater_stop();
    fn sendAck(reqid: u32, response: Option<u32>);
    fn sendAlarmReached(alarmId: u32, tick: u32, stopOnAlarm: bool);
}

pub type GateUpdateCollection = HashMap<String, (GatePtr, HashMap<String, Vec3vl>)>;

#[wasm_bindgen]
struct RustEngine {
    interval: u32,
    tick: u32,
    graphs: HashMap<String, GraphPtr>,
    monitors: HashMap<u32, MonitorParams>,
    monitor_checks: HashMap<u32, Vec3vl>,
    alarms: HashMap<u32, AlarmParams>,
    alarm_queue: HashMap<u32, HashSet<u32>>,
    queue: HashMap<u32, GateUpdateCollection>,
    pq: BTreeSet<u32>,
    to_update: HashMap<String, (GatePtr, HashSet<String>)>,
}

#[wasm_bindgen]
impl RustEngine {
    pub fn new() -> RustEngine {
        RustEngine { 
            interval: 10,
            tick: 0,
            graphs: HashMap::new(),
            monitors: HashMap::new(),
            monitor_checks: HashMap::new(),
            alarms: HashMap::new(),
            alarm_queue: HashMap::new(),
            queue: HashMap::new(),
            pq: BTreeSet::new(),
            to_update: HashMap::new()
        }
    }

    pub fn interval(&mut self, ms: u32) {
        self.interval = ms;
    }

    #[wasm_bindgen(js_name = getInterval)]
    pub fn get_interval(&self) -> u32 {
        self.interval
    }

    #[wasm_bindgen(js_name = updateGates)]
    pub fn update_gates(&mut self, reqid: u32, send_updates: bool) -> Result<(), String> {
        let count = self.update_gates_priv()?;
        if send_updates { self.send_updates_priv()?; }
        self.post_monitors()?;
        self.send_ack(reqid, Some(count));
        Ok(())
    }

    #[wasm_bindgen(js_name = _updateGates)]
    pub fn update_gates_priv(&mut self) -> Result<u32, String> {
        Ok(match self.pq.first() {
            Some(k) => {
                if *k == self.tick {
                    self.update_gates_next_priv()?
                } else {
                    self.tick = self.tick.wrapping_add(1);
                    0
                }
            },
            None => {
                self.tick = self.tick.wrapping_add(1);
                0
            }
        })
    }

    #[wasm_bindgen(js_name = updateGatesNext)]
    pub fn update_gates_next(&mut self, reqid: u32, send_updates: bool) -> Result<(), String> {
        let count = self.update_gates_next_priv()?;
        if send_updates { self.send_updates_priv()?; }
        self.post_monitors()?;
        self.send_ack(reqid, Some(count));
        Ok(())
    }

    fn update_gates_next_priv(&mut self) -> Result<u32, String> {
        let k = match self.pq.pop_first() {
            Some(p) => p,
            None => return Err("No events has been queued".to_string())
        };
        self.tick = k;
        let mut count = 0;

        while let Some(q) = self.queue.remove(&k) {
            for (gate, sigs) in q.values() {
                let result = gate.borrow_mut().do_operation(sigs)?;
                if result.is_clock() {
                    self.enqueue(gate);
                }
                
                self.set_gate_output_signals_priv(gate, result)?;
                count += 1;
            }

            if self.queue.contains_key(&k) {
                self.pq.pop_first();
            }
        }
        
        self.tick = self.tick.wrapping_add(1);
        Ok(count)
    }

    pub fn ping(&mut self, reqid: u32, send_updates: bool) -> Result<(), String> {
        if send_updates { self.send_updates_priv()?; }
        self.send_ack(reqid, None);
        Ok(())
    }

    #[wasm_bindgen(js_name = addGraph)]
    pub fn add_graph(&mut self, id: String) {
        self.graphs.insert(id.clone(), Graph::new(id));
    }

    #[wasm_bindgen(js_name = addLink)]
    pub fn add_link(&mut self, graph_id: String, link_id: String, from: TargetParams, to: TargetParams) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?;
        let source_target = LinkTarget { id: from.get_id(), port: from.get_port(), magnet: from.get_magnet() };
        let target_target = LinkTarget { id: to.get_id(), port: to.get_port(), magnet: to.get_magnet() };

        graph.borrow_mut().add_link(link_id, source_target.clone(), target_target.clone())?;

        let source_gate = graph.borrow().get_gate(&source_target.id)?;
        let target_gate = graph.borrow().get_gate(&target_target.id)?;

        let sig = source_gate.borrow().get_output(&source_target.port)?;

        self.set_gate_input_signal_priv(target_gate, target_target.port, sig)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = addGate)]
    pub fn add_gate(&mut self, graph_id: String, gate_id: String, gate_params: JsGateParams, port_params: Vec<PortParams>) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?.clone();

        graph.borrow_mut().add_gate(graph.clone(), gate_id.clone(), gate_params, port_params)?;
        self.enqueue(&graph.borrow().get_gate(&gate_id)?);
        Ok(())
    }

    #[wasm_bindgen(js_name = addSubcircuit)]
    pub fn add_subcircuit(&mut self, graph_id: String, gate_id: String, subgraph_id: String, io_map: Vec<IOmap>) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?;
        let gate = graph.borrow().get_gate(&gate_id)?;
        let subgraph = self.get_graph(subgraph_id)?.clone();

        gate.borrow_mut().set_subgraph(subgraph.clone());

        subgraph.borrow_mut().set_subcircuit(gate.clone());
        let mut map = HashMap::new();
        for i in io_map {
            let port = i.get_port();
            let io_id = i.get_io_id();

            map.insert(port.clone(), io_id.clone());
            let io = subgraph.borrow().get_gate(&io_id)?;
            match gate.borrow().get_port_dir(&port)? {
                IoDir::In => self.set_gate_output_signal_priv(&io, "out".to_string(), gate.borrow().get_input(&port)?)?,
                IoDir::Out => self.set_gate_output_signal_priv(&gate, port, io.borrow().get_input(&"in".to_string())?)?,
            };
        }

        gate.borrow_mut().set_subgraph_iomap(map);
        Ok(())
    }

    #[wasm_bindgen(js_name = removeLink)]
    pub fn remove_link(&mut self, graph_id: String, link_id: String) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?;
        let link = graph.borrow_mut().remove_link(&link_id)?;

        let target_gate = graph.borrow().get_gate(&link.target.id)?;
        let sig = Vec3vl::xes(target_gate.borrow().get_input(&link.target.port)?.bits);

        self.set_gate_input_signal_priv(target_gate, link.target.port, sig)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = removeGate)]
    pub fn remove_gate(&mut self, graph_id: String, gate_id: String) -> Result<(), String> {
        self.get_graph(graph_id)?.borrow_mut().remove_gate(&gate_id)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = observeGraph)]
    pub fn observe_graph(&mut self, graph_id: String) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?.clone();
        graph.borrow_mut().observe();

        for gate in graph.borrow().gate_iter() {
            for (port, dir) in gate.borrow().iodirs_iter() {
                if *dir == IoDir::Out {
                    self.mark_update_priv(gate, port.clone());
                }
            }
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = unobserveGraph)]
    pub fn unobserve_graph(&mut self, graph_id: String) -> Result<(), String> {
        self.get_graph(graph_id)?.borrow_mut().unobserve();
        Ok(())
    }

    #[wasm_bindgen(js_name = changeInput)]
    pub fn change_input(&mut self, graph_id: String, gate_id: String, sig: JsVec3vl) -> Result<(), String> {
        let graph = self.get_graph(graph_id)?;
        let gate = graph.borrow().get_gate(&gate_id)?;
        self.set_gate_output_signal_priv(
            &gate, 
            String::from("out"), 
            Vec3vl::from_clonable(sig)
        )?;
        Ok(())
    }

    // change param

    #[wasm_bindgen(js_name = manualMemChange)]
    pub fn manual_mem_change(&mut self, graph_id: String, gate_id: String, addr: u32, data: JsVec3vl) -> Result<(), String> {
        let gate = self.get_graph(graph_id)?.borrow().get_gate(&gate_id)?;
        gate.borrow_mut().set_memory(addr, Vec3vl::from_clonable(data))?;
        self.enqueue(&gate);
        Ok(())
    }

    pub fn monitor(&mut self, graph_id: String, gate_id: String, port: String, monitor_id: u32, params: JsMonitorParams) -> Result<(), String> {
        let gate = self.get_graph(graph_id)?.borrow().get_gate(&gate_id)?;
        let monitor_params = MonitorParams::new(params, gate.clone(), port.clone());

        if monitor_params.trigger_values.is_none() {
            let sig = gate.borrow().get_output(&port)?;
            postMonitorValue(monitor_id, self.tick, sig.bits, sig.avec, sig.bvec, None, None);
        }
        
        self.monitors.insert(monitor_id, monitor_params);
        gate.borrow_mut().monitor(port, monitor_id);
        Ok(())
    }

    pub fn unmonitor(&mut self, monitor_id: u32) -> Result<(), String> {
        if let Some(monitor) = self.monitors.remove(&monitor_id) {
            monitor.gate.borrow_mut().unmonitor(&monitor.port, monitor_id);
            self.monitor_checks.remove(&monitor_id);
        }
        Ok(())
    }

    pub fn alarm(&mut self, tick: u32, alarm_id: u32, data: JsAlarmStruct) {
        if tick <= self.tick { return; }

        self.alarms.insert(alarm_id, AlarmParams::new(data, tick));
        self.alarm_queue.entry(tick).or_default().insert(alarm_id);
        self.pq.insert(tick - 1);
        self.queue.entry(tick-1).or_default();
    }

    pub fn unalarm(&mut self, alarm_id: u32) {
        let alarm = match self.alarms.remove(&alarm_id) {
            Some(a) => a,
            None => { return; }
        };

        let aq = self.alarm_queue.get_mut(&alarm.tick).unwrap();
        aq.remove(&alarm_id);
        if aq.is_empty() {
            self.alarm_queue.remove(&alarm.tick);
        } 
    }

    fn enqueue(&mut self, gate: &GatePtr) {
        let k = self.tick.wrapping_add(gate.borrow().get_propagation());
        let sq = self.queue.entry(k).or_insert_with(|| {
            self.pq.insert(k);
            HashMap::new()
        });

        let name = gate.borrow().get_graph().borrow().get_id() + &gate.borrow().get_id();
        sq.insert(name, (gate.clone(), gate.borrow().get_inputs().clone()));
    }

    #[wasm_bindgen(js_name = _postMonitors)]
    pub fn post_monitors(&mut self) -> Result<(), String> {
        let monitors = self.monitor_checks.clone();
        self.monitor_checks = HashMap::new();

        for (monitor_id, sig) in monitors {
            let params = self.get_monitor(monitor_id)?;
            let mut triggered = true;
            if let Some(trigger_values) = params.trigger_values {
                triggered = trigger_values.iter().any(|v| *v == sig);
            }

            if triggered {
                if params.one_shot { self.unmonitor(monitor_id)?; }
                if params.synchronous { self.send_updates_priv()?; }
                postMonitorValue(monitor_id, self.tick, sig.bits, sig.avec, sig.bvec, Some(params.stop_on_trigger), Some(params.one_shot));
                if params.stop_on_trigger { updater_stop(); }
            }
        }

        if self.alarm_queue.contains_key(&self.tick) {
            let aq = self.alarm_queue.remove(&self.tick).unwrap();
            for alarm_id in aq {
                let alarm = self.alarms.remove(&alarm_id).unwrap();
                if alarm.synchronous { self.send_updates_priv()?; }
                sendAlarmReached(alarm_id, self.tick, alarm.stop_on_alarm);
                if alarm.stop_on_alarm { updater_stop(); }
            }
        }

        Ok(())
    }

    fn set_gate_output_signals_priv(&mut self, gate: &GatePtr, sigs: ReturnValue) -> Result<(), String> {
        if let Some(sig) = sigs.get_out() {
            self.set_gate_output_signal_priv(gate, "out".to_string(), sig)?;
        }

        for (port, sig) in sigs.signals_iter() {
            self.set_gate_output_signal_priv(gate, port.clone(), sig.clone())?;
        };

        Ok(())
    } 

    fn set_gate_output_signal_priv(&mut self, gate: &GatePtr, port: String, sig: Vec3vl) -> Result<(), String> {
        let old_sig = gate.borrow().get_output(&port)?;
        if old_sig == sig { return Ok(()); }

        gate.borrow_mut().set_output(port.clone(), sig.clone());
        self.mark_update_priv(gate, port.clone());

        let tgts = gate.borrow().get_targets(&port)?;
        for target in tgts {
            let target_gate = gate.borrow().get_graph().borrow().get_gate(&target.id)?;
            self.set_gate_input_signal_priv(target_gate, target.port, sig.clone())?;
        }
        
        for monitor_id in gate.borrow().get_monitors(&port) {
            self.monitor_checks.insert(*monitor_id, sig.clone());
        }

        Ok(())
    }

    fn set_gate_input_signal_priv(&mut self, target_gate: GatePtr, port: String, sig: Vec3vl) -> Result<(), String> {
        let old_sig = target_gate.borrow().get_input(&port)?;
        if old_sig == sig { return Ok(()); }
        target_gate.borrow_mut().set_input(port.clone(), sig.clone());

        if target_gate.borrow().is_subcircuit() {
            let subgraph = target_gate.borrow().get_subgraph()?; 
            let iomap = target_gate.borrow().get_subgraph_iomap_port(&port)?;
            let gate = subgraph.borrow().get_gate(&iomap)?;

            self.set_gate_output_signal_priv(&gate, "out".to_string(), sig)?;
        } else if target_gate.borrow().is_output() {
            let subgraph = target_gate.borrow().get_graph();
            if let Some(subcir) = subgraph.borrow().subcircuit() {
                let subcir_port = target_gate.borrow().get_subcir_net()?;
                self.set_gate_output_signal_priv(&subcir, subcir_port, sig)?;
            };
        } else {
            self.enqueue(&target_gate);
        }
        Ok(())
    }

    fn mark_update_priv(&mut self, gate: &GatePtr, port: String) {
        if !gate.borrow().get_graph().borrow().observed() {
            return;
        }

        let name = format!("{}{}", gate.borrow().graph_id(), gate.borrow().get_id());
        self.to_update.entry(name).or_insert((gate.clone(), HashSet::new())).1.insert(port);
    }

    #[wasm_bindgen(js_name = _sendUpdates)]
    pub fn send_updates_priv(&mut self) -> Result<(), String> {
        let mut updates= Vec::new();

        for (_gate_id, (gate, ports)) in self.to_update.iter() {
            let mut signals = Vec::new();

            for port in ports {
                let sig = gate.borrow().get_output(port)?;
                signals.push(PortUpdate { port: port.clone(), bits: sig.bits, avec: sig.avec, bvec: sig.bvec });
            }

            updates.push(UpdateStruct {
                graph_id: gate.borrow().graph_id(),
                gate_id: gate.borrow().get_id(),
                val: signals
            });
        }
        self.to_update = HashMap::new(); 
        sendUpdates(self.tick, self.has_pending_updates(), updates);
        Ok(())
    }

    #[wasm_bindgen(js_name = _sendAck)]
    pub fn send_ack(&self, reqid: u32, response: Option<u32>) {
        sendAck(reqid, response);
    }

    fn has_pending_updates(&self) -> bool {
        !self.queue.is_empty()
    }
}

impl RustEngine {
    fn get_graph(&self, graph_id: String) -> Result<&GraphPtr, String> {
        match self.graphs.get(&graph_id) {
            Some(g) => Ok(g),
            None => Err(format!("No graph with id {graph_id}"))
        }
    }

    fn get_monitor(&self, monitor_id: u32) -> Result<MonitorParams, String> {
        match self.monitors.get(&monitor_id) {
            Some(m) => Ok(m.clone()),
            None => Err(format!("No monitor id {} found", monitor_id))
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
struct UpdateStruct {
    pub graph_id: String,
    pub gate_id: String,
    pub val: Vec<PortUpdate>
}

#[derive(Clone)]
#[wasm_bindgen(getter_with_clone)]
struct PortUpdate {
    pub port: String,
    pub bits: u32,
    pub avec: Vec<u32>,
    pub bvec: Vec<u32>
}

#[derive(Clone)]
struct MonitorParams {
    pub trigger_values: Option<Vec<Vec3vl>>,
    pub stop_on_trigger: bool,
    pub one_shot: bool,
    pub synchronous: bool,
    pub gate: GatePtr,
    pub port: String
}

impl MonitorParams {
    pub fn new(params: JsMonitorParams, gate: GatePtr, port: String) -> MonitorParams {
        MonitorParams { 
            trigger_values: params.get_trigger_values().map(|v| 
                v.into_iter().map(Vec3vl::from_clonable).collect()
            ), 
            stop_on_trigger: params.get_stop_on_trigger().unwrap_or(false), 
            one_shot: params.get_one_shot().unwrap_or(false), 
            synchronous: params.get_synchronous().unwrap_or(false), 
            gate, 
            port 
        }
    }
}

#[derive(Clone)]
struct AlarmParams {
    pub tick: u32,
    pub stop_on_alarm: bool,
    pub synchronous: bool
}

impl AlarmParams {
    pub fn new(params: JsAlarmStruct, tick: u32) -> AlarmParams {
        AlarmParams { 
            tick, 
            stop_on_alarm: params.get_stop_on_alarm(), 
            synchronous: params.get_synchronous() 
        }
    }
}