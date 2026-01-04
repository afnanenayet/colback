use colback::ColbackView;

#[derive(ColbackView, Eq, PartialEq)]
struct SomeStructOpts {
    row_a: u32,
    row_c: Option<u16>,
    row_d: String,
}

fn main() {}
