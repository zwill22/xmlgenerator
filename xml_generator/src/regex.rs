use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use regex::Regex;

pub(crate) fn generate_regex(pattern: &Regex) -> Option<String> {
    let seed = 42;
    let mut rng = XorShiftRng::seed_from_u64(seed);

    let regex = match rand_regex::Regex::compile(pattern.as_str(), 100) {
        Ok(regex) => regex,
        Err(_) => return None,
    };

    let mut samples = (&mut rng)
        .sample_iter(&regex)
        .take(1000)
        .collect::<Vec<String>>();

    samples.sort();

    samples.first().map(|s| s.to_string())
}
