use jiff::Timestamp;

/// Where handlers get "now" from. With the `demo` feature, `SYSSLOR_DEMO_NOW` can freeze
/// it so that demo data and screenshots look the same whenever they are taken.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    #[cfg(feature = "demo")]
    fixed: Option<Timestamp>,
}

impl Clock {
    pub fn from_env() -> anyhow::Result<Self> {
        #[cfg(feature = "demo")]
        if let Ok(value) = std::env::var("SYSSLOR_DEMO_NOW") {
            return Ok(Self {
                fixed: Some(parse_fixed(&value)?),
            });
        }
        Ok(Self::default())
    }

    pub fn now(self) -> Timestamp {
        #[cfg(feature = "demo")]
        if let Some(fixed) = self.fixed {
            return fixed;
        }
        Timestamp::now()
    }
}

/// Accepts a zoned datetime like `2026-10-10T20:30[Europe/Stockholm]` or an RFC 3339
/// timestamp.
#[cfg(feature = "demo")]
fn parse_fixed(value: &str) -> anyhow::Result<Timestamp> {
    use anyhow::Context;

    if let Ok(zoned) = value.parse::<jiff::Zoned>() {
        return Ok(zoned.timestamp());
    }
    value
        .parse()
        .with_context(|| format!("SYSSLOR_DEMO_NOW {value:?} is not a valid datetime"))
}

#[cfg(all(test, feature = "demo"))]
mod tests {
    use super::*;

    #[test]
    fn parses_zoned_and_utc() {
        let expected: Timestamp = "2026-10-10T18:30:00Z".parse().unwrap();
        assert_eq!(
            parse_fixed("2026-10-10T20:30[Europe/Stockholm]").unwrap(),
            expected
        );
        assert_eq!(parse_fixed("2026-10-10T18:30:00Z").unwrap(), expected);
    }

    #[test]
    fn rejects_garbage_with_the_value_in_the_message() {
        let error = parse_fixed("tomorrow").unwrap_err();
        assert!(format!("{error:#}").contains("\"tomorrow\""), "{error:#}");
    }

    #[test]
    fn fixed_clock_stands_still() {
        let fixed = parse_fixed("2026-10-10T18:30:00Z").unwrap();
        let clock = Clock { fixed: Some(fixed) };
        assert_eq!(clock.now(), fixed);
    }
}
