use std::cmp::Ordering;

pub type GroupId = i64;

/// A named set of tasks within a project, such as a room in a home.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
}

/// Sorts groups alphabetically, ignoring case. Ties (duplicate names) fall back to id so
/// the order is stable across requests.
pub fn sort_groups(groups: &mut [Group]) {
    groups.sort_by(compare);
}

fn compare(a: &Group, b: &Group) -> Ordering {
    // Code point order, so Swedish å/ä/ö end up as ä < å < ö. Good enough without a
    // collation dependency.
    a.name
        .to_lowercase()
        .cmp(&b.name.to_lowercase())
        .then(a.id.cmp(&b.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(id: GroupId, name: &str) -> Group {
        Group {
            id,
            name: name.to_owned(),
        }
    }

    fn ids(groups: &[Group]) -> Vec<GroupId> {
        groups.iter().map(|group| group.id).collect()
    }

    #[test]
    fn sorts_by_name_ignoring_case() {
        let mut groups = vec![
            group(1, "kitchen"),
            group(2, "Bathroom"),
            group(3, "attic"),
            group(4, "Kitchen garden"),
        ];
        sort_groups(&mut groups);
        assert_eq!(ids(&groups), [3, 2, 1, 4]);
    }

    #[test]
    fn duplicate_names_fall_back_to_id() {
        let mut groups = vec![group(3, "Hall"), group(1, "hall"), group(2, "Hall")];
        sort_groups(&mut groups);
        assert_eq!(ids(&groups), [1, 2, 3]);
    }
}
