use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Formatter;

#[derive(Debug, serde::Deserialize)]
pub struct NameRecord {
    name: String,
    #[serde(rename = "clean name")]
    clean_name: String,
    syllables: String,
    count: usize,
    #[serde(rename = "first part")]
    first_part: Option<usize>,
    gender: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Name {
    pub name: String,
    pub clean_name: String,
    pub syllables: Vec<String>,
    pub guaranteed_parts: Vec<PartEntry>,
    pub possible_parts: Vec<PartEntry>,
    pub gender: char,
}

impl Name {
    pub fn from_record(record: NameRecord) -> Self {
        let syllables = record
            .syllables
            .split(".")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let guaranteed_parts = if let Some(fp) = record.first_part {
            vec![
                PartEntry::first((&syllables[..fp]).join(""), fp),
                PartEntry::second((&syllables[fp..]).join(""), syllables.len() - fp),
            ]
        } else {
            vec![]
        };

        Self {
            name: record.name,
            clean_name: record.clean_name,
            syllables,
            guaranteed_parts,
            possible_parts: vec![],
            gender: record.gender,
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
    let mut female_names = 0;
    let mut male_names = 0;
    let mut names = vec![];
    for record in rdr.deserialize() {
        let record: NameRecord = record.unwrap();
        names.push(Name::from_record(record));
    }
    for n in &mut names {
        if n.gender == 'm' {
            male_names += 1;
        } else {
            female_names += 1;
        }
        if n.guaranteed_parts.len() > 0 {
            continue;
        }
        if n.syllables.len() == 1 {
            n.possible_parts
                .push(PartEntry::lone(n.syllables[0].clone(), 1));
            continue;
        }
        for f in 1..n.syllables.len() {
            let first = (&n.syllables[..f]).join("");
            let second = (&n.syllables[f..]).join("");
            n.possible_parts.push(PartEntry::first(first, f));
            n.possible_parts
                .push(PartEntry::second(second, n.syllables.len() - f));
        }
    }
    let mut by_syllable = HashMap::new();
    for (idx, n) in names.iter().enumerate() {
        for (i, s) in n.syllables.iter().enumerate() {
            let entry =
                by_syllable
                    .entry(s.clone())
                    .or_insert((vec![], vec![], vec![], 0., 0.));
            if n.gender == 'm' {
                entry.3 += 1.;
            } else if n.gender == 'f' {
                entry.4 += 1.;
            } else {
                entry.3 += 0.5;
                entry.4 += 0.5;
            }
            if i == 0 {
                entry.0.push(idx);
            } else if i == n.syllables.len() - 1 {
                entry.1.push(idx);
            } else {
                entry.2.push(idx);
            }
        }
    }
    let mut syllable_occurrence = by_syllable
        .iter()
        .map(|(s, v)| (s.clone(), v.0.len(), v.1.len(), v.2.len(), v.3, v.4))
        .collect::<Vec<_>>();
    syllable_occurrence.sort_by_key(|a| a.1);
    syllable_occurrence.reverse();
    let mut by_part = HashMap::new();
    for (i, n) in names.iter().enumerate() {
        for p in &n.guaranteed_parts {
            let entry = by_part
                .entry((p.value.clone(), p.len))
                .or_insert((vec![], vec![], 0., 0.));
            if n.gender == 'm' {
                entry.2 += 1.;
            } else if n.gender == 'f' {
                entry.3 += 1.;
            } else {
                entry.2 += 0.5;
                entry.3 += 0.5;
            }
            match p.position {
                0 => entry.0.push((i, 1.)),
                1 => entry.1.push((i, 1.)),
                _ => {
                    entry.0.push((i, 0.5));
                    entry.1.push((i, 0.5));
                }
            }
        }
        for p in &n.possible_parts {
            let entry = by_part
                .entry((p.value.clone(), p.len))
                .or_insert((vec![], vec![], 0., 0.));
            if n.gender == 'm' {
                entry.2 += 1.;
            } else if n.gender == 'f' {
                entry.3 += 1.;
            } else {
                entry.2 += 0.5;
                entry.3 += 0.5;
            }
            let v = 2. / (n.possible_parts.len() as f32);
            match p.position {
                0 => entry.0.push((i, v)),
                1 => entry.1.push((i, v)),
                _ => {
                    entry.0.push((i, v / 2.));
                    entry.1.push((i, v / 2.));
                }
            }
        }
    }
    let part_occurrence = by_part
        .iter()
        .map(|(s, v)| {
            (
                s.0.clone(),
                s.1,
                v.0.iter().map(|e| e.1).sum::<f32>(),
                v.1.iter().map(|e| e.1).sum::<f32>(),
                v.2,
                v.3,
            )
        })
        .collect::<Vec<_>>();

    let gender_ratio = male_names as f32 / female_names as f32;

    let mut wtr = csv::Writer::from_path("syllable_data.csv".to_string()).unwrap();
    let mut syllable_records = vec![];
    for (s, first, second, middle, male, female) in &syllable_occurrence {
        let female = female * gender_ratio;
        let gender_ratio = male / (female + male);
        let str = s.clone();
        let value = *first + *second + *middle;
        let names = by_syllable
            .get(s)
            .map(|v| {
                v.0.iter()
                    .chain(v.1.iter())
                    .chain(v.2.iter())
                    .map(|i| names[*i].name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let record = OutputRecord {
            segment_kind: SegmentKind::Syllable,
            str,
            overall_val: value as f32,
            start_val: *first as f32,
            end_val: *second as f32,
            middle_val: *middle as f32,
            names: names.join(";"),
            gender_ratio,
        };
        wtr.serialize(&record).unwrap();
        syllable_records.push(record);
    }

    wtr.flush().unwrap();
    let mut wtr = csv::Writer::from_path("word_data.csv".to_string()).unwrap();
    let mut part_records = vec![];
    for (s, i, first, second, male, female) in &part_occurrence {
        let female = female * gender_ratio;
        let gender_ratio = male / (female + male);
        let str = s.clone();
        let value = *first + *second;
        let names = by_part
            .get(&(s.clone(), *i))
            .map(|v| {
                v.0.iter()
                    .chain(v.1.iter())
                    .map(|i| names[i.0].name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let record = OutputRecord {
            segment_kind: SegmentKind::Part,
            str,
             overall_val: value,
             start_val: *first,
             end_val: *second,
             middle_val: 0.,
            names: names.join(";"),
            gender_ratio,
        };
        wtr.serialize(&record).unwrap();
        part_records.push(record);
    }
    wtr.flush().unwrap();
    (syllable_records, part_records, names)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutputRecord {
    pub segment_kind: SegmentKind,
    pub str: String,
    pub overall_val: f32,
    pub start_val: f32,
    pub middle_val: f32,
    pub end_val: f32,
    pub names: String,
    pub gender_ratio: f32,
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct PositionalData {
    pub overall: f32,
    pub start: f32,
    pub middle: f32,
    pub end: f32,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum SegmentKind {
    Part,
    Syllable,
    Apostrophe
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NameSegment {
    pub segment_kind : SegmentKind,
    pub str: String,
    pub derived_names: Vec<String>,
    pub positional_data: PositionalData,
    pub gender_ratio: f32,
}

impl NameSegment {
    pub fn apostrophe() -> NameSegment {
        Self {
            segment_kind : SegmentKind::Apostrophe,
            str: "'".to_string(),
            derived_names: vec![],
            positional_data: PositionalData::default(),
            gender_ratio: 0.,
        }
    }
}

impl std::fmt::Display for NameSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.segment_kind {
            SegmentKind::Apostrophe => "'",
            _ => self.str.as_str(),
        })
    }
}

impl From<&OutputRecord> for NameSegment {
    fn from(record: &OutputRecord) -> Self {
        Self {
            segment_kind: record.segment_kind,
            str: record.str.clone(),
            derived_names: record.names.split(";").map(String::from).collect(),
            positional_data: PositionalData {
                overall: record.overall_val,
                start: record.start_val,
                middle: record.middle_val,
                end: record.end_val,
            },
            gender_ratio: record.gender_ratio,
        }
    }
}