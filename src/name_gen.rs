use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use crate::data_processing::{generate_data, Name, OutputRecord};
use crate::util::capitalize;

const CUTOFF : f32 = 2.0;
const VOWELS : [char; 5] = ['a', 'e', 'i', 'o', 'u'];
const WEIGHT: f32 = 1.;

pub fn generate_names_from_parts(parts: &Vec<OutputRecord>, _: &Vec<Name>, amount: usize, omit_reserved: bool) -> Vec<(String, Vec<String>)> {
    let reserved = vec!["jin","fon","zen","zul"];
    let parts = if omit_reserved {
        parts.into_iter().filter(|v| !reserved.iter().any(|s| *s == v.str.as_str())).collect::<Vec<_>>()
    } else {
        parts.into_iter().collect::<Vec<_>>()
    };
    let mut first = parts.iter().filter(|o| o.value_first > CUTOFF).collect::<Vec<_>>();
    let mut first_weights = WeightedIndex::new(first.iter().map(|o| o.value_first.powf(WEIGHT))).unwrap();
    let mut second = parts.iter().filter(|o| o.value_second > CUTOFF).collect::<Vec<_>>();
    let mut second_weights = WeightedIndex::new(second.iter().map(|o| o.value_second.powf(WEIGHT))).unwrap();
    let mut rng = rand::rng();
    let mut results = Vec::new();
    for _ in 0..amount {
        let first = &first[first_weights.sample(&mut rng)];
        let second = second[second_weights.sample(&mut rng)];
        let name = format!("{}'{}", capitalize(&first.str), second.str);
        let mut references = first.names.split(";").chain(second.names.split(";")).map(|v| v.to_string()).collect::<Vec<_>>();
        references.sort();
        references.dedup();
        results.push((name,references));
    }
    results
}


pub fn example_name_gen() {
    let (syllables, parts, names) = generate_data();
    let existing_names = names.iter().map(|n| n.syllables.join("")).collect::<Vec<_>>();
    let reserved = vec!["jin","fon","zen","zul"];
    let parts = parts.into_iter().filter(|v| !reserved.iter().any(|s| *s == v.str.as_str())).collect::<Vec<_>>();
    let mut first = parts.iter().filter(|o| o.value_first > CUTOFF).collect::<Vec<_>>();
    let mut first_weights = WeightedIndex::new(first.iter().map(|o| o.value_first.powf(WEIGHT))).unwrap();
    let mut second = parts.iter().filter(|o| o.value_second > CUTOFF).collect::<Vec<_>>();
    let mut second_weights = WeightedIndex::new(second.iter().map(|o| o.value_second.powf(WEIGHT))).unwrap();
    let mut rng = rand::rng();
    for _ in 0..50 {
        loop {
            let first = &first[first_weights.sample(&mut rng)];
            let second = second[second_weights.sample(&mut rng)];
            let name = format!("{}'{}", capitalize(&first.str), second.str);
            if existing_names.contains(&format!("{}{}", first.str, second.str)) {
                // println!("{} already exists", name);
                continue;
            }
            println!("{}", name);
            break;
        }
    }
}