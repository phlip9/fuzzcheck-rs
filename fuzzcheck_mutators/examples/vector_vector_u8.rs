extern crate fuzzcheck_mutators;
use fuzzcheck::Mutator;
use fuzzcheck_mutators::integer::*;
use fuzzcheck_mutators::vector::*;

fn main() {
    type M = VecMutator<VecMutator<U8Mutator>>;
    let mut m = M::default();

    let mut x = vec![vec![2, 89, 1, 0, 4, 2], vec![3], vec![4, 9, 0, 0, 0, 2, 5], vec![]];
    let mut x_cache = m.cache_from_value(&x);
    let mut x_step = m.mutation_step_from_value(&x);

    let mut results: Vec<Vec<Vec<u8>>> = vec![];
    for _ in 0..100 {
        let token = m.mutate(&mut x, &mut x_cache, &mut x_step, 100.0);
        results.push(x.clone());

        m.unmutate(&mut x, &mut x_cache, token);
    }
    for x in results.iter() {
        println!("{:?}", x);
    }

    results.clear();

    for i in 0..20 {
        results.push(m.arbitrary(i, 100.0).0);
    }

    println!("{:?}", results);
}
