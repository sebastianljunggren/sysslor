//! Pure scheduling logic: cadence, due dates and sorting. No I/O.

// Not wired into the API until milestone 2.
#![allow(dead_code, unused_imports)]

mod cadence;
mod schedule;

pub use cadence::{Cadence, CadenceUnit};
pub use schedule::{Schedule, ScheduledTask, Task, TaskId, sort_tasks};
