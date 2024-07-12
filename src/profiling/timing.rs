use std::{collections::HashMap, sync::Mutex};

use log::info;
use serde::*;

lazy_static::lazy_static! {
    pub static ref SUBTRACT_INTENTS: Mutex<bool> = Mutex::new(false);

    pub static ref INTENTS_USED: Mutex<u32> = Mutex::new(0);
    pub static ref PROFILER_INTENT_TRACKING: Mutex<HashMap<String, u32>> = Mutex::new(HashMap::new());
}

pub type IdentStr = &'static str;

static mut TRACE: Option<Trace> = None;

pub fn start_trace(clock: Box<dyn Fn() -> u64>) {
    unsafe {
        if TRACE.is_some() {
            panic!("Expected trace to be not be set!");
        }

        TRACE = Some(Trace::new(clock));
    }
}

pub fn stop_trace() -> Option<Trace> {
    unsafe { TRACE.take() }
}

pub fn get_mut_trace() -> Option<&'static mut Trace> {
    unsafe { TRACE.as_mut() }
}

#[derive(Serialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    events: Vec<Event>,
    #[serde(skip_serializing)]
    clock: Box<dyn Fn() -> u64>,
}

impl Trace {
    fn new(clock: Box<dyn Fn() -> u64>) -> Trace {
        Trace {
            clock,
            events: Vec::new(),
        }
    }

    pub fn get_time(&self) -> u64 {
        (self.clock)()
    }
}

#[derive(Serialize)]
struct TracingEvent {
    #[serde(rename = "name")]
    name: IdentStr,
    #[serde(rename = "pid")]
    process_id: u32,
    #[serde(rename = "intents_used")]
    intents_used: u32,
    #[serde(rename = "tid")]
    thread_id: u32,
    #[serde(rename = "ts")]
    timestamp: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct BeginEvent {
    name: IdentStr,
    time: u64,
}

impl Into<TracingEvent> for BeginEvent {
    fn into(self) -> TracingEvent {
        TracingEvent {
            name: self.name,
            process_id: 0,
            intents_used: 0,
            thread_id: 0,
            timestamp: self.time,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(into = "TracingEvent")]
struct EndEvent {
    name: IdentStr,
    intents_used: u32,
    time: u64,
}

impl From<EndEvent> for TracingEvent {
    fn from(val: EndEvent) -> Self {
        TracingEvent {
            name: val.name,
            process_id: 0,
            thread_id: 0,
            intents_used: val.intents_used,
            timestamp: val.time,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "ph")]
enum Event {
    #[serde(rename = "B")]
    Begin(BeginEvent),
    #[serde(rename = "E")]
    End(EndEvent),
}

#[must_use = "The guard is immediately dropped after instantiation. This is probably not
what you want! Consider using a `let` binding to increase its lifetime."]
pub struct SpanGuard {
    name: IdentStr,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        end(self.name);
    }
}

pub fn start_guard<S: Into<IdentStr>>(name: S) -> SpanGuard {
    let name = name.into();
    start(name);
    SpanGuard { name }
}

fn start<S: Into<IdentStr>>(name: S) {
    if let Some(trace) = get_mut_trace() {
        let time = trace.get_time();
        let name: &str = name.into();

        let time = if *SUBTRACT_INTENTS.lock().unwrap() {
            // Insert the intents we have used so far.
            // This way, we can track the amount of intents used by a function.
            PROFILER_INTENT_TRACKING
                .lock()
                .unwrap()
                .insert(name.to_string(), *INTENTS_USED.lock().unwrap());

            let subtract_amount = ((*INTENTS_USED.lock().unwrap() as f64 * 0.2) * 1000.0) as u64;

            time - subtract_amount
        } else {
            time
        };

        let event = BeginEvent {
            name,
            time,
        };

        trace.events.push(Event::Begin(event));
    }
}

fn end<S: Into<IdentStr>>(name: S) {
    let name = name.into();

    if let Some(trace) = get_mut_trace() {
        let time = trace.get_time();

        let mut intents_used = 0;
        if let Some(intents_at_call) = PROFILER_INTENT_TRACKING.lock().unwrap().remove(name) {
            let intents_after_call = *INTENTS_USED.lock().unwrap();

            let intents_used_by_function = intents_after_call - intents_at_call;
            if intents_used_by_function > 0 {
                intents_used = intents_used_by_function;
            }
        }

        let time = if *SUBTRACT_INTENTS.lock().unwrap() {
            //let subtract_amount = PROFILER_SUBTRACT.lock().unwrap().clone() as u64;
            let subtract_amount = ((*INTENTS_USED.lock().unwrap() as f64 * 0.2) * 1000.0) as u64;

            time - subtract_amount
        } else {
            time
        };

        let event = EndEvent {
            name,
            intents_used,
            time,
        };

        trace.events.push(Event::End(event));
    }
}
