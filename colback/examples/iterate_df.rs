use colback::ColbackView;
use polars::{df, frame::DataFrame};

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStruct {
    row_a: u32,
    row_b: bool,
}

fn generate_df() -> DataFrame {
    let len = 100000;
    let row_a_col: Vec<u32> = (0..len).collect();
    let row_b_col: Vec<bool> = row_a_col.iter().map(|i| i % 2 == 0).collect();

    let df = df! [
        "row_a" => row_a_col,
        "row_b" => row_b_col,
    ]
    .unwrap();
    df
}

fn main() {
    let df = generate_df();
    let view = SomeStruct::view(&df).unwrap();
    let mut res = 0;

    for row_proxy in view.iter() {
        let row_proxy = unsafe { row_proxy.unwrap_unchecked() };
        if row_proxy.row_b {
            res += row_proxy.row_a
        }
    }
    println!("res: {}", res);
}
