use rand::Rng;
use rand::rngs::ThreadRng;

#[derive(PartialEq)]
pub(crate) struct Datetime {
    pattern: String,
}

fn generate_year(rng: &mut ThreadRng) -> i32 {
    let mut year = rng.random_range(1..10000);
    if year == 0 {
        return generate_year(rng)
    }
    if rng.random::<f32>() < 0.01 {
        year *= -1;
    }

    year
}

fn generate_day(rng: &mut ThreadRng, year: i32, month: i32) -> i32 {
    match month {
        9 | 4 | 6 | 11 => rng.random_range(1..=30),
        2 => {
            if year % 4 == 0 {
                rng.random_range(1..=29)
            } else {
                rng.random_range(1..=28)
            }
        },
        _ => rng.random_range(1..=31),
    }
}

impl Datetime {

    pub(crate) fn generate(&self, rng: &mut ThreadRng) -> Option<String> {
        if self.pattern.is_empty() {
            return None;
        }

        let mut output = self.pattern.clone();

        let year = generate_year(rng);
        if self.pattern.contains("%Y") {
            let year_str = format!("{:04}", year);

            output = output.replace("%Y", &year_str);
        }

        let month = rng.random_range(1..=12);
        if self.pattern.contains("%m") {
            let month_str = format!("{:02}", month);
            output = output.replace("%m", &month_str);
        }

        if self.pattern.contains("%d") {
            let day = generate_day(rng, year, month);
            let day_str = format!("{:02}", day);
            output = output.replace("%d", &day_str);
        }

        if self.pattern.contains("%H") {
            let hour = rng.random_range(0..24);
            let hour_str = format!("{:02}", hour);
            output = output.replace("%H", &hour_str);
        }

        if self.pattern.contains("%M") {
            let minute = rng.random_range(0..60);
            let minute_str = format!("{:02}", minute);
            output = output.replace("%m", &minute_str);
        }

        if self.pattern.contains("%S") {
            let second = rng.random_range(0..60);
            let second_str = format!("{:02}", second);
            output = output.replace("%S", &second_str);
        }

        if output.contains("%") {
            panic!("Raw time string not replaced")
        }

        Some(output)
    }
}

impl From<&str> for Datetime {
    fn from(s: &str) -> Self {
        Datetime { pattern: s.to_string() }
    }
}

impl std::fmt::Display for Datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}