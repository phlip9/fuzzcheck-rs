use std::{collections::HashMap, path::PathBuf};

use super::CodeCoverageSensor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CoverageMap {
    functions: Vec<Function>,
}

#[derive(Serialize, Deserialize)]
pub struct Function {
    name: String,
    file: String,
    counters: Vec<Counter>,
}

#[derive(Serialize, Deserialize)]
pub struct Region {
    lines: (usize, usize),
    cols: (usize, usize),
}

#[derive(Serialize, Deserialize)]
pub struct Counter {
    id: usize,
    regions: Vec<Region>,
}

impl CodeCoverageSensor {
    #[no_coverage]
    pub(crate) fn coverage_map(&self) -> CoverageMap {
        let mut idx = 0;
        let functions = self
            .coverage
            .iter()
            .map(
                #[no_coverage]
                |coverage| {
                    let f_record = &coverage.function_record;
                    assert!(f_record.filenames.len() == 1);
                    let name = f_record.name_function.clone();
                    let mut regions_by_file = HashMap::<PathBuf, Vec<Counter>>::new();

                    let mut indices_and_regions = vec![];
                    for (e, region) in &f_record.expressions {
                        if e.add_terms.len() == 1 && e.sub_terms.is_empty() {
                            indices_and_regions.push((idx, region));
                            idx += 1;
                        }
                    }
                    for (e, region) in &f_record.expressions {
                        if !(e.add_terms.len() == 1 && e.sub_terms.is_empty()) && !e.add_terms.is_empty() {
                            indices_and_regions.push((idx, region));
                            idx += 1;
                        }
                    }

                    for (idx, regions) in indices_and_regions {
                        let file_idx = f_record
                            .file_id_mapping
                            .filename_indices
                            .iter()
                            .position(
                                #[no_coverage]
                                |idx| *idx == regions[0].filename_index,
                            )
                            .unwrap();
                        let file = f_record.filenames[file_idx].clone();
                        let counter = Counter {
                            id: idx,
                            regions: regions
                                .iter()
                                .map(|region| Region {
                                    lines: (region.line_start, region.line_end),
                                    cols: (region.col_start, region.col_end),
                                })
                                .collect(),
                        };
                        regions_by_file.entry(file).or_default().push(counter);
                    }
                    let (file, counters) = regions_by_file.into_iter().next().unwrap();
                    Function {
                        name,
                        file: file.to_str().unwrap().to_owned(),
                        counters,
                    }
                },
            )
            .collect();
        CoverageMap { functions }
    }
}
