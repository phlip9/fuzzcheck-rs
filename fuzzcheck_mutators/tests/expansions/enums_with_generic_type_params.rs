use fuzzcheck_mutators::fuzzcheck_traits::Mutator;
use fuzzcheck_mutators::DefaultMutator;

#[derive(Clone, Debug, DefaultMutator)]
pub enum X<T> {
    A(T),
    B(Vec<T>),
}

#[test]
fn test_compile() {
    let m = X::<Vec<u8>>::default_mutator();
    let (value, _cache): (X<Vec<u8>>, _) = m.random_arbitrary(100.0);

    match &value {
        X::A(_x) => {}
        X::B(_y) => {}
    }
    println!("{:?}", value);
}
