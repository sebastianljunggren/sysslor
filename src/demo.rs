//! Demo data for trying out the app and taking screenshots. Only compiled with the `demo`
//! feature, which release images never enable.

use anyhow::{Context, bail};
use jiff::civil::Time;
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp};
use sqlx::SqlitePool;
use uuid::Builder;

use crate::db::completions::{self, Completion};
use crate::db::projects::ProjectId;
use crate::db::tasks::{self, TaskFields};
use crate::db::{self, executors, groups, projects};
use crate::domain::{Cadence, CadenceUnit, GroupColor};

/// The project created by the first migration.
const PROJECT_ID: ProjectId = 1;

const PROJECT_NAME: &str = "Home";

const EXECUTORS: [&str; 3] = ["Alex", "Sam", "Robin"];

const GROUPS: [(&str, GroupColor); 4] = [
    ("Kitchen", GroupColor::Orange),
    ("Bathroom", GroupColor::Cyan),
    ("Living room", GroupColor::Green),
    ("Bedroom", GroupColor::Violet),
];

struct DemoTask {
    name: &'static str,
    /// Index into `GROUPS`.
    group: Option<usize>,
    cadence: Cadence,
    priority: u8,
    /// Calendar days since the last completion, `None` if never completed.
    days_ago: Option<i64>,
}

const fn days(amount: u32) -> Cadence {
    Cadence {
        amount,
        unit: CadenceUnit::Days,
    }
}

const fn weeks(amount: u32) -> Cadence {
    Cadence {
        amount,
        unit: CadenceUnit::Weeks,
    }
}

const fn demo_task(
    name: &'static str,
    group: Option<usize>,
    cadence: Cadence,
    priority: u8,
    days_ago: Option<i64>,
) -> DemoTask {
    DemoTask {
        name,
        group,
        cadence,
        priority,
        days_ago,
    }
}

// Listed in the order they are sorted. Only the first two are overdue, and the top six mix
// groups, cadences and due dates so a screenshot shows the variety. The range hood filter
// is the only never-completed task, so it is always on top. Vacuum is the only task with a
// priority; it is not overdue, so the priority does not move it. Executors rotate by index,
// so the people who did the top tasks differ too.
const TASKS: [DemoTask; 13] = [
    demo_task("Clean range hood filter", Some(0), weeks(4), 0, None),
    demo_task("Change bed sheets", Some(3), weeks(2), 0, Some(15)),
    demo_task("Vacuum", Some(2), weeks(1), 3, Some(6)),
    demo_task("Scrub shower", Some(1), weeks(2), 0, Some(11)),
    demo_task("Wipe counters", Some(0), days(3), 0, Some(2)),
    demo_task("Take out recycling", None, weeks(1), 0, Some(4)),
    demo_task("Water plants", Some(2), days(4), 0, Some(2)),
    demo_task("Clean fridge", Some(0), weeks(2), 0, Some(6)),
    demo_task("Clean toilet", Some(1), weeks(1), 0, Some(3)),
    demo_task("Air out duvets", Some(3), weeks(8), 0, Some(20)),
    demo_task("Wash bath mats", Some(1), weeks(2), 0, Some(5)),
    demo_task("Descale kettle", Some(0), weeks(4), 0, Some(3)),
    demo_task("Dust shelves", Some(2), weeks(3), 0, Some(0)),
];

/// Seeds the database at `SYSSLOR_DATABASE_URL` and exits, see `seed`.
pub async fn run(database_url: &str, time_zone: &TimeZone, now: Timestamp) -> anyhow::Result<()> {
    let pool = db::connect(database_url).await?;
    seed(&pool, now, time_zone).await?;
    tracing::info!(
        "seeded {database_url} with {} groups and {} tasks",
        GROUPS.len(),
        TASKS.len()
    );
    Ok(())
}

/// Fills an empty database with demo data, with completions relative to `now`. Refuses to
/// touch a database that already has data, so demo data never mixes with real chores.
async fn seed(pool: &SqlitePool, now: Timestamp, tz: &TimeZone) -> anyhow::Result<()> {
    let is_empty = groups::list(pool, PROJECT_ID).await?.is_empty()
        && tasks::list_active(pool, PROJECT_ID).await?.is_empty()
        && executors::list(pool).await?.is_empty();
    if !is_empty {
        bail!("refusing to seed demo data into a database that already has data");
    }

    projects::rename(pool, PROJECT_ID, PROJECT_NAME)
        .await?
        .with_context(|| format!("project {PROJECT_ID} does not exist"))?;

    let mut executor_ids = Vec::new();
    for name in EXECUTORS {
        executor_ids.push(executors::create(pool, name).await?.id);
    }

    let mut group_ids = Vec::new();
    for (name, color) in GROUPS {
        let group = groups::create(pool, PROJECT_ID, name, Some(color))
            .await?
            .with_context(|| format!("project {PROJECT_ID} does not exist"))?;
        group_ids.push(group.id);
    }

    let today = now.to_zoned(tz.clone()).date();
    for (index, demo) in TASKS.iter().enumerate() {
        let fields = TaskFields {
            name: demo.name.to_owned(),
            cadence: demo.cadence,
            priority: demo.priority,
            group_id: demo.group.map(|group| group_ids[group]),
        };
        let tasks::Write::Done(_, task) = tasks::create(pool, PROJECT_ID, &fields, now).await?
        else {
            bail!("failed to create demo task {:?}", demo.name);
        };

        let Some(days_ago) = demo.days_ago else {
            continue;
        };
        // An evening completion looks natural, but today's must not be in the future.
        let completed_at = today
            .checked_sub(Span::new().days(days_ago))?
            .to_datetime(Time::constant(18, 30, 0, 0))
            .to_zoned(tz.clone())?
            .timestamp()
            .min(now);
        let millis = u64::try_from(completed_at.as_millisecond())
            .with_context(|| format!("completion time {completed_at} is before 1970"))?;
        // Deterministic "random" bits keep reruns on a fresh database reproducible.
        let random = [u8::try_from(index)?; 10];
        let completion = Completion {
            id: Builder::from_unix_timestamp_millis(millis, &random).into_uuid(),
            task_id: task.id,
            executor_id: executor_ids[index % executor_ids.len()],
            completed_at,
        };
        completions::upsert(pool, &completion).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use jiff::Zoned;

    use super::*;
    use crate::domain::{sort_tasks, test_collator};

    fn stockholm() -> TimeZone {
        TimeZone::get("Europe/Stockholm").unwrap()
    }

    fn now() -> Timestamp {
        "2026-10-10T09:00[Europe/Stockholm]"
            .parse::<Zoned>()
            .unwrap()
            .timestamp()
    }

    #[sqlx::test]
    async fn range_hood_filter_is_on_top(pool: SqlitePool) {
        seed(&pool, now(), &stockholm()).await.unwrap();

        let project = projects::get(&pool, PROJECT_ID).await.unwrap().unwrap();
        assert_eq!(project.name, "Home");

        let groups = groups::list(&pool, PROJECT_ID).await.unwrap();
        let colors: HashSet<_> = groups
            .iter()
            .map(|group| format!("{:?}", group.color))
            .collect();
        assert_eq!(groups.len(), 4);
        assert_eq!(colors.len(), 4);

        let tasks = tasks::list_active(&pool, PROJECT_ID).await.unwrap();
        let last = completions::latest_per_task(&pool, PROJECT_ID)
            .await
            .unwrap()
            .into_iter()
            .map(|(task_id, completion)| (task_id, completion.completed_at))
            .collect();
        let sorted = sort_tasks(tasks, &last, now(), &stockholm(), &test_collator("en"));
        assert_eq!(sorted.len(), TASKS.len());
        let names: Vec<_> = sorted.iter().map(|t| t.task.name.as_str()).collect();
        let listed: Vec<_> = TASKS.iter().map(|t| t.name).collect();
        assert_eq!(names, listed);
        let overdue = sorted.iter().filter(|t| t.schedule.is_overdue()).count();
        assert_eq!(overdue, 2);
    }

    #[sqlx::test]
    async fn refuses_a_database_with_data(pool: SqlitePool) {
        seed(&pool, now(), &stockholm()).await.unwrap();
        assert!(seed(&pool, now(), &stockholm()).await.is_err());
        assert_eq!(
            groups::list(&pool, PROJECT_ID).await.unwrap().len(),
            GROUPS.len()
        );
    }
}
