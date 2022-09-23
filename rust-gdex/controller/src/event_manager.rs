// IMPORTS

// crate

// gdex
use gdex_types::transaction::{Event, ExecutionEvent, ExecutionResultBody};

// external
use prost::Message;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::{Arc, Mutex};

// INTERFACE

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventManager {
    pub current_execution_result: ExecutionResultBody,
}

impl EventManager {
    pub fn new() -> Self {
        EventManager {
            current_execution_result: ExecutionResultBody::new(),
        }
    }

    pub fn reset(&mut self) {
        self.current_execution_result = ExecutionResultBody::new();
    }

    pub fn push(&mut self, event: ExecutionEvent) {
        self.current_execution_result.events.push(event);
    }

    pub fn emit(&mut self) -> ExecutionResultBody {
        let mut execution_result = ExecutionResultBody::new();
        mem::swap(&mut self.current_execution_result, &mut execution_result);
        execution_result
    }
}

// TRAIT

pub trait EventEmitter {
    fn get_event_manager(&mut self) -> &mut Arc<Mutex<EventManager>>;

    fn emit_event<T: Event + Message + std::default::Default>(&mut self, event: &T) {
        self.get_event_manager()
            .lock()
            .unwrap()
            .push(ExecutionEvent::new(event));
    }
}
