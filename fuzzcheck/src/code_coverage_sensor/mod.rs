//! Code coverage analysis

mod leb128;
mod llvm_coverage;
use crate::Feature;
use std::convert::TryFrom;
use std::path::Path;
use std::{collections::HashMap, ops::RangeInclusive};

use self::llvm_coverage::{get_counters, get_prf_data, read_covmap, Coverage, LLVMCovSections};

/// Records the code coverage of the program and converts it into `Feature`s
/// that the `pool` can understand.
pub struct CodeCoverageSensor {
    pub coverage: Vec<Coverage>,
    pub index_ranges: Vec<RangeInclusive<usize>>,
}

impl CodeCoverageSensor {
    #[no_coverage]
    pub(crate) fn new<E, K>(exclude: E, keep: K) -> Self
    where
        E: Fn(&Path) -> bool,
        K: Fn(&Path) -> bool,
    {
        let exec = std::env::current_exe().expect("could not read current executable");
        let LLVMCovSections {
            covfun,
            covmap,
            prf_names,
        } = llvm_coverage::get_llvm_cov_sections(&exec).expect("could not find all relevant LLVM coverage sections");
        let prf_data = unsafe { get_prf_data() };
        let covmap = read_covmap(&covmap, &mut 0).expect("failed to parse LLVM covmap");
        let covfun = llvm_coverage::read_covfun(&covfun, &mut 0).expect("failed to parse LLVM covfun");

        let prf_names = llvm_coverage::read_prf_names(&prf_names, &mut 0).expect("failed to parse LLVM prf_names");
        let mut map = HashMap::new();
        for prf_name in prf_names {
            let name_md5 = md5::compute(prf_name.as_bytes());
            let name_md5 = i64::from_le_bytes(<[u8; 8]>::try_from(&name_md5[0..8]).unwrap());
            map.insert(name_md5, prf_name);
        }

        let covfun = llvm_coverage::process_function_records(covfun, map, &covmap);
        let prf_data = llvm_coverage::read_prf_data(prf_data, &mut 0).expect("failed to parse LLVM prf_data");

        let mut coverage = unsafe { Coverage::new(covfun, prf_data, get_counters()) }
            .expect("failed to properly link the different LLVM coverage sections");
        coverage.drain_filter(|coverage| coverage.single_counters.len() + coverage.expression_counters.len() <= 1);
        Coverage::filter_function_by_files(&mut coverage, exclude, keep);

        let mut index_ranges = Vec::new();

        let mut index = 0;
        for coverage in coverage.iter() {
            let next_index = index + coverage.single_counters.len() + coverage.expression_counters.len();
            index_ranges.push(index..=next_index - 1);
            index = next_index;
        }
        assert_eq!(coverage.len(), index_ranges.len());
        CodeCoverageSensor { coverage, index_ranges }
    }
    #[no_coverage]
    pub(crate) unsafe fn start_recording(&self) {}
    #[no_coverage]
    pub(crate) unsafe fn stop_recording(&self) {}
    #[no_coverage]
    pub(crate) unsafe fn iterate_over_collected_features<F>(&mut self, coverage_index: usize, mut handle: F)
    where
        F: FnMut(Feature),
    {
        let CodeCoverageSensor { coverage, index_ranges } = self;
        let coverage = coverage.get_unchecked(coverage_index);
        let mut index = *index_ranges.get_unchecked(coverage_index).start();

        let single = *coverage.single_counters.get_unchecked(0);
        if *single == 0 {
            return;
        } else {
            handle(Feature::new(index, *single));
        }
        index += 1;
        for &single in coverage.single_counters.iter().skip(1) {
            if *single != 0 {
                handle(Feature::new(index, *single));
            }
            index += 1;
        }
        for exp in &coverage.expression_counters {
            let computed = exp.compute();
            if computed != 0 {
                handle(Feature::new(index, computed));
            }
            index += 1;
        }
    }
    #[no_coverage]
    pub(crate) unsafe fn clear(&mut self) {
        for coverage in &self.coverage {
            let slice = std::slice::from_raw_parts_mut(coverage.start_counters, coverage.counters_len);
            for c in slice.iter_mut() {
                *c = 0;
            }
        }
    }
}
