use std::cmp::Ordering;
use std::collections::HashMap;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, serde::Deserialize)]
pub struct NameRecord {
    name: String,
    #[serde(rename = "clean name")]
    clean_name: String,
    syllables: String,
    count: usize,
    #[serde(rename = "first part")]
    first_part: Option<usize>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub clean_name: String,
    pub syllables: Vec<String>,
    pub guaranteed_parts: Vec<PartEntry>,
    pub possible_parts: Vec<PartEntry>,
}

impl Name {
    pub fn from_record(record: NameRecord) -> Self {
        let syllables = record.syllables.split(".").map(|s| s.to_string()).collect::<Vec<String>>();
        let guaranteed_parts = if let Some(fp) = record.first_part {
            vec![PartEntry::first((&syllables[..fp]).join(""), fp), PartEntry::second((&syllables[fp..]).join(""), syllables.len() - fp)]
        } else {
            vec![]
        };

        Self {
            name: record.name,
            clean_name: record.clean_name,
            syllables,
            guaranteed_parts,
            possible_parts: vec![],
        }
    }
}

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub struct PartEntry {
    pub value: String,
    pub len: usize,
    pub position: usize,
}

impl PartEntry {
    pub fn first(value: String, len: usize) -> PartEntry {
        Self {
            value,
            len,
            position: 0,
        }
    }

    pub fn second(value: String, len: usize) -> PartEntry {
        Self {
            value,
            len,
            position: 1,
        }
    }

    pub fn lone(value: String, len: usize) -> PartEntry {
        Self {
            value,
            len,
            position: 2,
        }
    }
}

pub fn generate_data() -> (Vec<OutputRecord>, Vec<OutputRecord>, Vec<Name>) {
    let mut rdr = csv::Reader::from_path("data/syllables.csv".to_string()).unwrap();
    let mut names = vec![];
    for record in rdr.deserialize() {
        let record : NameRecord = record.unwrap();
        names.push(Name::from_record(record));
    }
    for n in &mut names {
        if n.guaranteed_parts.len() > 0  { continue; }
        if n.syllables.len() == 1 { n.possible_parts.push(PartEntry::lone(n.syllables[0].clone(), 1)); continue; }
        for f in 1..n.syllables.len() {
            let first = (&n.syllables[..f]).join("");
            let second = (&n.syllables[f..]).join("");
            n.possible_parts.push(PartEntry::first(first, f));
            n.possible_parts.push(PartEntry::second(second, n.syllables.len() - f));
        }
    }
    let mut by_syllable = HashMap::new();
    for (idx, n) in names.iter().enumerate() {
        for (i, s) in n.syllables.iter().enumerate() {
            let mut entry = by_syllable.entry(s.clone()).or_insert((vec![], vec![], vec![]));
            if i == 0 {
                entry.0.push(idx);
            } else if i == n.syllables.len() - 1 {
                entry.1.push(idx);
            } else {
                entry.2.push(idx);
            }
        }
    }
    let mut syllable_occurrence = by_syllable.iter().map(|(s, v)| (s.clone(), v.0.len(), v.1.len(), v.2.len())).collect::<Vec<_>>();
    syllable_occurrence.sort_by_key(|a| a.1);
    syllable_occurrence.reverse();
    let mut by_part = HashMap::new();
    for (i, n) in names.iter().enumerate() {
        for p in &n.guaranteed_parts {
            let mut e = by_part.entry((p.value.clone(), p.len)).or_insert((vec![], vec![]));
            match p.position {
                0 => e.0.push((i,1.)),
                1 => e.1.push((i,1.)),
                _ => {
                    e.0.push((i,0.5));
                    e.1.push((i,0.5));
                }
            }
        }
        for p in &n.possible_parts {
            let mut e = by_part.entry((p.value.clone(), p.len)).or_insert((vec![], vec![]));
            let v = 2./(n.possible_parts.len() as f32);
            match p.position {
                0 => e.0.push((i,v)),
                1 => e.1.push((i,v)),
                _ => {
                    e.0.push((i,v / 2.));
                    e.1.push((i,v / 2.));
                }
            }
        }
    }
    let part_occurrence = by_part.iter().map(|(s, v)| {
        (s.0.clone(), s.1,
         v.0.iter().map(|e| e.1).sum::<f32>(),
         v.1.iter().map(|e| e.1).sum::<f32>()
        )
    }).collect::<Vec<_>>();

    let mut wtr = csv::Writer::from_path("syllable_data.csv".to_string()).unwrap();

    let mut syllable_records = vec![];
    for (s, first, second, middle) in &syllable_occurrence {
        let str = s.clone();
        let value = *first + *second + *middle;
        let names = by_syllable.get(s).map(|v| v.0.iter().chain(v.1.iter()).chain(v.2.iter()).map(|i| names[*i].name.clone()).collect::<Vec<_>>()).unwrap_or_default();
        let record = OutputRecord {
            str,
            value : value as f32,
            value_first: *first as f32,
            value_second: *second as f32,
            value_middle: *middle as f32,
            names: names.join(";"),
        };
        wtr.serialize(&record).unwrap();
        syllable_records.push(record);
    }
    wtr.flush().unwrap();
    let mut wtr = csv::Writer::from_path("word_data.csv".to_string()).unwrap();

    let mut part_records = vec![];
    for (s, i,first, second) in &part_occurrence{
        let str = s.clone();
        let value = *first + *second;
        let names = by_part.get(&(s.clone(), *i)).map(|v| v.0.iter().chain(v.1.iter()).map(|i| names[i.0].name.clone()).collect::<Vec<_>>()).unwrap_or_default();
        let record = OutputRecord {
            str,
            value,
            value_first: *first,
            value_second: *second,
            value_middle: 0.,
            names: names.join(";"),
        };
        wtr.serialize(&record).unwrap();
        part_records.push(record);
    }
    wtr.flush().unwrap();
    (syllable_records, part_records, names)
}

#[derive(Serialize, Deserialize)]
pub struct OutputRecord {
    pub str: String,
    pub value: f32,
    pub value_first: f32,
    pub value_second: f32,
    pub value_middle: f32,
    pub names: String,
}
