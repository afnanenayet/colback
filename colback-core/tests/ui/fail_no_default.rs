use colback_core::ColbackView;

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStructOpts {
    row_a: u32,
    #[polars(null = "default")]
    row_b: bool,
    #[polars(null = "option")]
    row_c: Option<u16>,
    row_d: String,
}

fn main() {}
