use std::cmp::Ordering;
use std::collections::HashMap;

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp};

use super::{Cadence, Collator, GroupId};

pub type TaskId = i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub cadence: Cadence,
    /// 0-5, where 5 is the highest priority.
    pub priority: u8,
    pub group_id: Option<GroupId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Schedule {
    /// A task that has never been completed is due right away.
    NeverCompleted,
    Completed {
        last_completed: Timestamp,
        /// The calendar day (in the configured time zone) the task is due again.
        due: Date,
        /// Calendar days since the last completion divided by the cadence in days.
        /// `>= 1.0` means the task is due or overdue.
        urgency: f64,
    },
}

impl Schedule {
    fn new(
        last_completed: Option<Timestamp>,
        cadence: Cadence,
        now: Timestamp,
        tz: &TimeZone,
    ) -> Self {
        let Some(last_completed) = last_completed else {
            return Self::NeverCompleted;
        };
        let today = now.to_zoned(tz.clone()).date();
        let completed_on = last_completed.to_zoned(tz.clone()).date();
        // A completion "in the future" (clock skew between devices) counts as today.
        let elapsed_days = (today - completed_on).get_days().max(0);
        let cadence_days = cadence.in_days();
        let due = Span::new()
            .try_days(i64::from(cadence_days))
            .ok()
            .and_then(|span| completed_on.checked_add(span).ok())
            .unwrap_or(Date::MAX);
        Self::Completed {
            last_completed,
            due,
            urgency: f64::from(elapsed_days) / f64::from(cadence_days),
        }
    }

    /// Urgency for sorting: tasks that were never completed are infinitely urgent.
    fn urgency(&self) -> f64 {
        match self {
            Self::NeverCompleted => f64::INFINITY,
            Self::Completed { urgency, .. } => *urgency,
        }
    }

    pub fn is_overdue(&self) -> bool {
        self.urgency() >= 1.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTask {
    pub task: Task,
    pub schedule: Schedule,
}

/// Sorts tasks so that the ones needing attention come first:
///
/// 1. Overdue tasks (`urgency >= 1`), by priority descending, then urgency descending.
/// 2. Remaining tasks, by urgency descending.
///
/// Remaining ties are broken by priority, then name (in the collator's alphabetical
/// order), then id, so the order is stable across devices.
pub fn sort_tasks(
    tasks: Vec<Task>,
    last_completions: &HashMap<TaskId, Timestamp>,
    now: Timestamp,
    tz: &TimeZone,
    collator: &Collator,
) -> Vec<ScheduledTask> {
    let mut scheduled: Vec<ScheduledTask> = tasks
        .into_iter()
        .map(|task| {
            let last = last_completions.get(&task.id).copied();
            let schedule = Schedule::new(last, task.cadence, now, tz);
            ScheduledTask { task, schedule }
        })
        .collect();
    scheduled.sort_by(|a, b| compare(a, b, collator));
    scheduled
}

fn compare(a: &ScheduledTask, b: &ScheduledTask, collator: &Collator) -> Ordering {
    let by_urgency = || b.schedule.urgency().total_cmp(&a.schedule.urgency());
    let by_priority = || b.task.priority.cmp(&a.task.priority);
    let tie_break = || {
        by_priority()
            .then_with(|| collator.compare(&a.task.name, &b.task.name))
            .then_with(|| a.task.id.cmp(&b.task.id))
    };

    match (a.schedule.is_overdue(), b.schedule.is_overdue()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) => by_priority().then_with(by_urgency).then_with(tie_break),
        (false, false) => by_urgency().then_with(tie_break),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_collator;
    use jiff::Zoned;
    use jiff::civil::date;

    fn stockholm() -> TimeZone {
        TimeZone::get("Europe/Stockholm").unwrap()
    }

    /// Parses e.g. `"2026-10-05T23:30[Europe/Stockholm]"`.
    fn at(s: &str) -> Timestamp {
        s.parse::<Zoned>().unwrap().timestamp()
    }

    fn task(id: TaskId, cadence: Cadence, priority: u8) -> Task {
        Task {
            id,
            name: format!("task {id}"),
            cadence,
            priority,
            group_id: None,
        }
    }

    fn schedule_of(cadence: Cadence, last: &str, now: &str) -> Schedule {
        Schedule::new(Some(at(last)), cadence, at(now), &stockholm())
    }

    fn ids(sorted: &[ScheduledTask]) -> Vec<TaskId> {
        sorted.iter().map(|t| t.task.id).collect()
    }

    #[test]
    fn never_completed_is_overdue() {
        let schedule = Schedule::new(None, Cadence::weeks(1), Timestamp::UNIX_EPOCH, &stockholm());
        assert_eq!(schedule, Schedule::NeverCompleted);
        assert!(schedule.is_overdue());
    }

    #[test]
    fn monday_evening_completion_is_due_next_monday() {
        let last = "2026-10-05T21:45[Europe/Stockholm]"; // a Monday

        let sunday = schedule_of(
            Cadence::weeks(1),
            last,
            "2026-10-11T23:59[Europe/Stockholm]",
        );
        let Schedule::Completed { due, urgency, .. } = sunday else {
            panic!("expected completed");
        };
        assert_eq!(due, date(2026, 10, 12));
        assert_eq!(urgency, 6.0 / 7.0);
        assert!(!sunday.is_overdue());

        // Monday morning: less than 7 × 24 hours have passed, but it is due.
        let monday = schedule_of(
            Cadence::weeks(1),
            last,
            "2026-10-12T06:00[Europe/Stockholm]",
        );
        assert!(matches!(monday, Schedule::Completed { urgency: 1.0, .. }));
        assert!(monday.is_overdue());
    }

    #[test]
    fn days_are_counted_in_the_configured_time_zone() {
        // 23:30 and 00:10 local are 21:30 and 22:10 UTC on the same UTC day.
        let last = at("2026-10-05T23:30[Europe/Stockholm]");
        let now = at("2026-10-06T00:10[Europe/Stockholm]");

        let local = Schedule::new(Some(last), Cadence::days(1), now, &stockholm());
        assert!(
            matches!(local, Schedule::Completed { due, urgency: 1.0, .. } if due == date(2026, 10, 6))
        );

        let utc = Schedule::new(Some(last), Cadence::days(1), now, &TimeZone::UTC);
        assert!(matches!(utc, Schedule::Completed { urgency: 0.0, .. }));
    }

    #[test]
    fn spring_forward_does_not_shift_due_date() {
        // DST starts 2026-03-29 in Sweden, that day is 23 hours long.
        let daily = schedule_of(
            Cadence::days(1),
            "2026-03-28T23:30[Europe/Stockholm]",
            "2026-03-29T00:30[Europe/Stockholm]",
        );
        assert!(
            matches!(daily, Schedule::Completed { due, urgency: 1.0, .. } if due == date(2026, 3, 29))
        );

        let weekly = schedule_of(
            Cadence::weeks(1),
            "2026-03-23T23:00[Europe/Stockholm]",
            "2026-03-30T00:00[Europe/Stockholm]",
        );
        assert!(weekly.is_overdue());
    }

    #[test]
    fn fall_back_does_not_shift_due_date() {
        // DST ends 2026-10-25 in Sweden, that day is 25 hours long.
        let weekly = schedule_of(
            Cadence::weeks(1),
            "2026-10-19T23:30[Europe/Stockholm]",
            "2026-10-25T23:59[Europe/Stockholm]",
        );
        assert!(matches!(weekly, Schedule::Completed { due, .. } if due == date(2026, 10, 26)));
        assert!(!weekly.is_overdue());
    }

    #[test]
    fn completion_in_the_future_counts_as_today() {
        let schedule = schedule_of(
            Cadence::days(2),
            "2026-10-07T12:00[Europe/Stockholm]",
            "2026-10-05T12:00[Europe/Stockholm]",
        );
        assert!(matches!(schedule, Schedule::Completed { urgency: 0.0, .. }));
    }

    #[test]
    fn huge_cadence_does_not_overflow() {
        let schedule = schedule_of(
            Cadence::weeks(u32::MAX),
            "2026-10-05T12:00[Europe/Stockholm]",
            "2026-10-05T12:00[Europe/Stockholm]",
        );
        assert!(matches!(
            schedule,
            Schedule::Completed {
                due: Date::MAX,
                urgency: 0.0,
                ..
            }
        ));
    }

    #[test]
    fn overdue_tasks_sort_by_priority_then_urgency() {
        let now = at("2026-10-10T12:00[Europe/Stockholm]");
        let tasks = vec![
            task(1, Cadence::days(1), 1), // urgency 2
            task(2, Cadence::days(1), 3), // urgency 2
            task(3, Cadence::days(1), 3), // urgency 5
            task(4, Cadence::days(1), 3), // never completed
            task(5, Cadence::days(1), 0), // urgency 9
        ];
        let last = HashMap::from([
            (1, at("2026-10-08T12:00[Europe/Stockholm]")),
            (2, at("2026-10-08T12:00[Europe/Stockholm]")),
            (3, at("2026-10-05T12:00[Europe/Stockholm]")),
            (5, at("2026-10-01T12:00[Europe/Stockholm]")),
        ]);

        let sorted = sort_tasks(tasks, &last, now, &stockholm(), &test_collator("sv"));
        assert_eq!(ids(&sorted), [4, 3, 2, 1, 5]);
    }

    #[test]
    fn remaining_tasks_sort_by_urgency_ignoring_priority() {
        let now = at("2026-10-10T12:00[Europe/Stockholm]");
        let tasks = vec![
            task(1, Cadence::days(10), 5), // urgency 0.1
            task(2, Cadence::days(10), 0), // urgency 0.5
            task(3, Cadence::days(4), 2),  // urgency 0.75
        ];
        let last = HashMap::from([
            (1, at("2026-10-09T12:00[Europe/Stockholm]")),
            (2, at("2026-10-05T12:00[Europe/Stockholm]")),
            (3, at("2026-10-07T12:00[Europe/Stockholm]")),
        ]);

        let sorted = sort_tasks(tasks, &last, now, &stockholm(), &test_collator("sv"));
        assert_eq!(ids(&sorted), [3, 2, 1]);
    }

    #[test]
    fn overdue_low_priority_beats_high_priority_not_yet_due() {
        let now = at("2026-10-10T12:00[Europe/Stockholm]");
        let tasks = vec![
            task(1, Cadence::days(7), 5), // urgency 6/7
            task(2, Cadence::days(7), 0), // urgency 1, due today
        ];
        let last = HashMap::from([
            (1, at("2026-10-04T12:00[Europe/Stockholm]")),
            (2, at("2026-10-03T12:00[Europe/Stockholm]")),
        ]);

        let sorted = sort_tasks(tasks, &last, now, &stockholm(), &test_collator("sv"));
        assert_eq!(ids(&sorted), [2, 1]);
    }

    #[test]
    fn ties_are_broken_by_priority_then_name_then_id() {
        let now = at("2026-10-10T12:00[Europe/Stockholm]");
        let mut tasks = vec![
            task(1, Cadence::days(7), 1),
            task(2, Cadence::days(7), 4),
            task(3, Cadence::days(7), 1),
            task(4, Cadence::days(7), 1),
        ];
        tasks[0].name = "Vacuum".into();
        tasks[2].name = "Dishes".into();
        tasks[3].name = "Dishes".into();
        let completed = at("2026-10-09T12:00[Europe/Stockholm]");
        let last = HashMap::from([
            (1, completed),
            (2, completed),
            (3, completed),
            (4, completed),
        ]);

        let sorted = sort_tasks(tasks, &last, now, &stockholm(), &test_collator("sv"));
        assert_eq!(ids(&sorted), [2, 3, 4, 1]);
    }

    #[test]
    fn name_ties_follow_the_collators_alphabet() {
        let now = at("2026-10-10T12:00[Europe/Stockholm]");
        let mut tasks = vec![task(1, Cadence::days(7), 0), task(2, Cadence::days(7), 0)];
        // Byte order would put Ä (U+00C4) before Å (U+00C5); Swedish has Å first.
        tasks[0].name = "Äta".into();
        tasks[1].name = "Åka".into();
        let completed = at("2026-10-09T12:00[Europe/Stockholm]");
        let last = HashMap::from([(1, completed), (2, completed)]);

        let sorted = sort_tasks(tasks, &last, now, &stockholm(), &test_collator("sv"));
        assert_eq!(ids(&sorted), [2, 1]);
    }
}
