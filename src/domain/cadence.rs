#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceUnit {
    Days,
    Weeks,
}

/// How often a task should be done, e.g. every 2 weeks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
    pub amount: u32,
    pub unit: CadenceUnit,
}

impl Cadence {
    #[cfg(test)]
    pub fn days(amount: u32) -> Self {
        Self {
            amount,
            unit: CadenceUnit::Days,
        }
    }

    #[cfg(test)]
    pub fn weeks(amount: u32) -> Self {
        Self {
            amount,
            unit: CadenceUnit::Weeks,
        }
    }

    /// The cadence in calendar days. Never less than 1, so it is safe to divide by.
    pub fn in_days(self) -> u32 {
        let days = match self.unit {
            CadenceUnit::Days => self.amount,
            CadenceUnit::Weeks => self.amount.saturating_mul(7),
        };
        days.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_are_seven_days() {
        assert_eq!(Cadence::weeks(2).in_days(), 14);
        assert_eq!(Cadence::days(3).in_days(), 3);
    }

    #[test]
    fn zero_is_treated_as_one_day() {
        assert_eq!(Cadence::days(0).in_days(), 1);
        assert_eq!(Cadence::weeks(0).in_days(), 1);
    }
}
