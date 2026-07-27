use crate::pattern::Pattern;
use crate::whitespace::WhiteSpace;
use rand::Rng;
use rand::rngs::{ StdRng };
use regex::Regex;

#[derive(PartialEq)]
pub(crate) struct Datetime {
    strf_time: String,
    pattern: Pattern,
}

fn generate_year(rng: &mut StdRng) -> i32 {
    let mut year = rng.random_range(1..10000);
    if year == 0 {
        return generate_year(rng);
    }
    if rng.random::<f32>() < 0.01 {
        year *= -1;
    }

    year
}

fn generate_day(rng: &mut StdRng, year: i32, month: i32) -> i32 {
    match month {
        9 | 4 | 6 | 11 => rng.random_range(1..=30),
        2 => {
            if year % 4 == 0 { rng.random_range(1..=29) } else { rng.random_range(1..=28) }
        }
        _ => rng.random_range(1..=31),
    }
}

impl Datetime {
    pub(crate) fn get_pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub(crate) fn generate(&self, rng: &mut StdRng) -> Option<String> {
        if self.strf_time.is_empty() {
            return None;
        }

        let mut output = self.strf_time.clone();

        let year = generate_year(rng);
        if self.strf_time.contains("%Y") {
            let year_str = format!("{:04}", year);

            output = output.replace("%Y", &year_str);
        }

        let month = rng.random_range(1..=12);
        if self.strf_time.contains("%m") {
            let month_str = format!("{:02}", month);
            output = output.replace("%m", &month_str);
        }

        if self.strf_time.contains("%d") {
            let day = generate_day(rng, year, month);
            let day_str = format!("{:02}", day);
            output = output.replace("%d", &day_str);
        }

        if self.strf_time.contains("%H") {
            let hour = rng.random_range(0..24);
            let hour_str = format!("{:02}", hour);
            output = output.replace("%H", &hour_str);
        }

        if self.strf_time.contains("%M") {
            let minute = rng.random_range(0..60);
            let minute_str = format!("{:02}", minute);
            output = output.replace("%M", &minute_str);
        }

        if self.strf_time.contains("%S") {
            let second = rng.random_range(0..60);
            let second_str = format!("{:02}", second);
            output = output.replace("%S", &second_str);
        }

        if output.contains("%") {
            panic!("Raw time string not replaced");
        }

        Some(output)
    }

    pub(crate) fn matches(&self, string: &str) -> bool {
        let full_pattern = format!("^{}$", self.pattern.get_pattern(true));
        let regex = Regex::new(&full_pattern).unwrap();
        regex.is_match(string)
    }
}

impl From<&str> for Datetime {
    fn from(s: &str) -> Self {
        const YEAR: &str =
            r"(-?(?:[0-9]{3}[1-9])|(?:[0-9]{2}[1-9][0-9])|(?:[0-9][1-9][0-9]{2})|(?:[1-9][0-9]{3}))";
        const MONTH: &str = r"(?:0[1-9]|1[0-2])";
        const DAY: &str = r"(?:0[1-9]|1[0-9]|2[0-9]|3[0-1])";

        const HOUR: &str = r"(?:0[1-9]|1[0-9]|2[0-3])";
        const MINUTE: &str = r"[0-5][0-9]";
        const SECOND: &str = r"[0-5][0-9]";

        let output = s
            .replace("%Y", YEAR)
            .replace("%m", MONTH)
            .replace("%d", DAY)
            .replace("%H", HOUR)
            .replace("%M", MINUTE)
            .replace("%S", SECOND);

        if output.contains("%") {
            panic!("Raw time string not replaced");
        }

        Datetime {
            strf_time: s.to_string(),
            pattern: Pattern::from_string(&output, &WhiteSpace::Collapse).unwrap(),
        }
    }
}

impl std::fmt::Display for Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}
