use colback_core::ColbackView;

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStruct {
    row_a: u32,
    row_b: bool,
}

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStructOpts {
    #[polars(null = "default", default = 1)]
    row_a: u32,
    row_b: bool,
    #[polars(null = "option")]
    row_c: Option<u16>,
    row_d: String,
}

fn main() {}
