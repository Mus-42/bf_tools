use bf_tools::{
    //bf,
    ins::BfCode,
    interpreter::{InterpCode, Interpreter},
    optimizer::*, bf,
};

fn main() {

    //TODO add cli tools app?
    //let ins: BfCode = bf!();
    let ins: BfCode = include_str!("../../target/out.bf").parse().unwrap();

    //println!("chars_len: {}", ins.chars_len());
    //println!("{}", &ins);

    let ins: OptCode = ins.into();

    //println!("opt_len: {}", ins.ins_len());
    //println!("{:?}", &ins);

    let ins = OptState::builder()
        .add_default_passes()
        .build()
        .run_passes(ins);

    //println!("opt_len: {}", ins.ins_len());
    //println!("{:?}", &ins);

    let ins = BfCode::from(ins);

    //println!("chars_len: {}", ins.chars_len());
    //println!("{}", &ins);

    let ins: InterpCode = ins.into();

    //println!("{}", &ins);

    //let mut bf_output = Vec::new();

    {
        let mut interpreter = Interpreter::builder()
            //.set_stdout(&mut bf_output)
            .build();
        interpreter.run(ins).unwrap();
        
        println!("\ntape: {:?}", &interpreter.tape);
        println!("\nptr: {:?}", &interpreter.data_pointer);
    }

    //println!("{}", std::str::from_utf8(&bf_output).unwrap_or_default());
    //println!("{:?}", &bf_output);

    //let mut interpreter = Interpreter::builder().build();
    //interpreter.run(ins).unwrap();
}