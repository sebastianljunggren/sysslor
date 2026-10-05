//! Pure scheduling logic: cadence, due dates and sorting. No I/O.

mod cadence;
mod group;
mod schedule;

pub use cadence::{Cadence, CadenceUnit};
pub use group::{Group, GroupColor, GroupId, sort_groups};
pub use schedule::{Schedule, ScheduledTask, Task, TaskId, sort_tasks};

/// Orders names the way the configured locale expects, e.g. Swedish å, ä, ö after z.
/// Its data is compiled in, so comparing does no I/O.
pub type Collator = icu_collator::CollatorBorrowed<'static>;

#[cfg(test)]
pub fn test_collator(locale: &str) -> Collator {
    let locale = icu_locale_core::Locale::try_from_str(locale).unwrap();
    Collator::try_new(locale.into(), Default::default()).unwrap()
}
