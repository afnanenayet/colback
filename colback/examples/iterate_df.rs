use colback::ColbackView;
use polars::{df, frame::DataFrame};
use rand::{
    Rng,
    distr::{Bernoulli, Uniform},
};

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStruct {
    row_a: u32,
    row_b: bool,
}

fn generate_df() -> DataFrame {
    println!("starting to generate df");
    let len = 1_000_000_000;
    let mut rng = rand::rng();
    let range = Uniform::new(0, 10000).unwrap();
    let bool_range = Bernoulli::new(0.7).unwrap();
    let row_a_col: Vec<u32> = (0..len).map(|_| rng.sample(&range)).collect();
    let row_b_col: Vec<bool> = (0..len).map(|_| rng.sample(&bool_range)).collect();

    let df = df! [
        "row_a" => row_a_col,
        "row_b" => row_b_col,
    ]
    .unwrap();
    println!("df height: {}, {:#?}", df.height(), df.schema());
    println!("returning");
    df
}

fn iterate() -> u64 {
    let df = generate_df();
    let view = SomeStruct::view(&df).unwrap();
    let mut res: u64 = 0;

    for row_proxy in view.iter() {
        let row_proxy = unsafe { row_proxy.unwrap_unchecked() };
        if row_proxy.row_b {
            res += row_proxy.row_b as u64;
        }
    }
    res
}

fn main() {
    let res = iterate();
    println!("res: {}", res);
}
