use super::Collator;

pub type GroupId = i64;

/// An accent from the Solarized palette, used to tell groups apart at a glance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupColor {
    Yellow,
    Orange,
    Red,
    Magenta,
    Violet,
    Blue,
    Cyan,
    Green,
}

/// A named set of tasks within a project, such as a room in a home.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub color: Option<GroupColor>,
}

/// Sorts groups alphabetically for the collator's locale. Ties (duplicate names) fall
/// back to id so the order is stable across requests.
pub fn sort_groups(groups: &mut [Group], collator: &Collator) {
    groups.sort_by(|a, b| collator.compare(&a.name, &b.name).then(a.id.cmp(&b.id)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_collator;

    fn group(id: GroupId, name: &str) -> Group {
        Group {
            id,
            name: name.to_owned(),
            color: None,
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
        sort_groups(&mut groups, &test_collator("en"));
        assert_eq!(ids(&groups), [3, 2, 1, 4]);
    }

    #[test]
    fn sorts_by_the_locales_alphabet() {
        let mut groups = vec![
            group(1, "Ört"),
            group(2, "Äng"),
            group(3, "Ås"),
            group(4, "Zon"),
            group(5, "Apa"),
        ];
        sort_groups(&mut groups, &test_collator("sv"));
        assert_eq!(ids(&groups), [5, 4, 3, 2, 1]);
        // English treats the dotted and ringed letters as variants of a and o.
        sort_groups(&mut groups, &test_collator("en"));
        assert_eq!(ids(&groups), [2, 5, 3, 1, 4]);
    }

    #[test]
    fn duplicate_names_fall_back_to_id() {
        let mut groups = vec![group(3, "Hall"), group(1, "hall"), group(2, "Hall")];
        sort_groups(&mut groups, &test_collator("en"));
        assert_eq!(ids(&groups), [1, 2, 3]);
    }
}
