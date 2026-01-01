use std::fmt::Formatter;
use crate::data_processing::{Name, SegmentKind, NameSegment};
use crate::util::{capitalize, ends_with_consonant, starts_with_consonant};
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;

const CUTOFF: f32 = 2.0;
const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
const WEIGHT: f32 = 1.;

fn generate_weights(
    list: &[&NameSegment],
    start: bool,
    end: bool,
    middle: bool,
) -> WeightedIndex<f32> {
    WeightedIndex::new(list.iter().map(|segment| {
        let mut value = 0.;
        let mut count: f32 = 0.;
        if start {
            count += 1.;
            value += segment.positional_data.start
        }
        if end {
            count += 1.;
            value += segment.positional_data.end
        }
        if middle {
            count += 1.;
            value += segment.positional_data.middle
        }
        count = count.max(1.);
        (value / count).powf(WEIGHT)
    }))
    .unwrap()
}


pub struct GeneratedName {
    name: String,
    pub elements: Vec<NameSegment>
}

impl GeneratedName {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            elements: Vec::new(),
        }
    }

    pub fn gender(&self) -> f32 {
        let mut count = 0;
        self.elements.iter().map(|s| {
            match s.segment_kind {
                SegmentKind::Apostrophe => 0.,
                _ => {
                    count += 1;
                    s.gender_ratio
                }
            }
        }).sum::<f32>() / (count as f32).max(1.)
    }

    pub fn bake(&mut self) {
        self.name = capitalize(&self.elements.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(""));
    }
}

impl std::fmt::Display for GeneratedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn generate_names_from_parts(
    parts: &Vec<NameSegment>,
    syllables: &Vec<NameSegment>,
    _: &Vec<Name>,
    amount: usize,
    omit_reserved: bool,
    length: f32,
) -> Vec<GeneratedName> {
    let reserved = vec!["jin", "fon", "zen", "zul"];
    let parts = if omit_reserved {
        parts
            .into_iter()
            .filter(|v| !reserved.iter().any(|s| *s == v.str.as_str()))
            .collect::<Vec<_>>()
    } else {
        parts.into_iter().collect::<Vec<_>>()
    };
    let part_weights = generate_weights(&parts, true, true, true);
    let first = parts
        .iter()
        .filter(|o| o.positional_data.start > CUTOFF)
        .copied()
        .collect::<Vec<_>>();
    let first_weights = generate_weights(&first, true, false, false);
    let second = parts
        .iter()
        .filter(|o| o.positional_data.end > CUTOFF)
        .copied()
        .collect::<Vec<_>>();
    let second_weights = generate_weights(&second, false, true, false);
    let middle = syllables
        .iter()
        .filter(|o| o.positional_data.middle > 0.)
        .collect::<Vec<_>>();
    let middle_weights = generate_weights(&middle, false, false, true);
    let consonant_bounds = middle
        .iter()
        .map(|s| {
            let cs = s.str.chars().collect::<Vec<char>>();
            (
                !VOWELS.contains(&cs[0]),
                !VOWELS.contains(&cs[cs.len() - 1]),
                s,
            )
        })
        .collect::<Vec<_>>();
    let open_start = consonant_bounds
        .iter()
        .filter(|(s, _, _)| !s)
        .map(|(_, _, o)| **o)
        .collect::<Vec<_>>();
    let open_end = consonant_bounds
        .iter()
        .filter(|(_, e, _)| !e)
        .map(|(_, _, o)| **o)
        .collect::<Vec<_>>();
    let open_start_weights = generate_weights(&open_start, false, false, true);
    let open_end_weights = generate_weights(&open_end, false, false, true);
    let mut rng = rand::rng();
    let mut generated_results = Vec::new();
    for _ in 0..amount {
        let mut generated_name = GeneratedName::new();
        let mut length = length;
        if length < 2. {
            if rng.random::<f32>() > (length - 1.) {
                let result = parts[part_weights.sample(&mut rng)];
                generated_name.elements.push(result.clone());
                generated_results.push(generated_name);
                continue;
            }
            length = 0.;
        } else {
            length -= 2.;
        }
        let first = first[first_weights.sample(&mut rng)];
        let second = second[second_weights.sample(&mut rng)];
        generated_name.elements.push(first.clone());
        generated_name.elements.push(NameSegment::apostrophe());
        generated_name.elements.push(second.clone());
        let mut start_pos = 0;
        let mut end_pos = 2;
        let mut syllable_insert = length;
        while rng.random::<f32>() < syllable_insert {
            let first_str = &generated_name.elements[start_pos].str;
            let second_str = &generated_name.elements[start_pos].str;
            let after_first = rng.random::<f32>() > 0.5;
            let allow_consonant_clusters = rng.random::<f32>() < 0.05;
            let syl = if after_first && ends_with_consonant(first_str) && !allow_consonant_clusters
            {
                open_start[open_start_weights.sample(&mut rng)]
            } else if !after_first
                && starts_with_consonant(&second_str)
                && !allow_consonant_clusters
            {
                open_end[open_end_weights.sample(&mut rng)]
            } else {
                &syllables[middle_weights.sample(&mut rng)]
            };
            if after_first {
                generated_name.elements.insert(start_pos + 1, syl.clone());
                start_pos += 1;
                end_pos += 1;
            } else {
                generated_name.elements.insert(end_pos, syl.clone());
            }
            let falloff = rng.random::<f32>() + 1.;
            syllable_insert /= falloff;
        }
        generated_results.push(generated_name);
    }
    generated_results.iter_mut().for_each(|n| n.bake());
    generated_results
}
