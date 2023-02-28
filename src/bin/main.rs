use bf_tools::{
    bf,
    optimizer::{passes::*, *},
};

fn main() {
    //TODO add cli tools app?
    let ins = bf!(+[>>+<<]++++++[-][>,]-><-);
    println!("{}", &ins);
    let ins = OptState::builder()
        .add_default_passes()
        .build()
        .run_passes(ins);

    println!("{}", &ins);
}
