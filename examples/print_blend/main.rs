use blend::Blend;
use libflate::gzip::Decoder;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    process::exit,
};

fn print_blend(blend_path: impl AsRef<Path>) -> Result<(), io::Error> {
    println!("{}", blend_path.as_ref().display());
    let mut file = File::open(blend_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data[0..7] != *b"BLENDER" {
        let mut decoder = Decoder::new(&data[..])?;
        let mut gzip_data = Vec::new();
        decoder.read_to_end(&mut gzip_data)?;

        data = gzip_data;
    }

    let blend = Blend::new(&data[..]);

    for o in blend.get_all_root_blocks() {
        print!("{}", o);
    }

    println!("done");

    Ok(())
}

pub fn main() -> Result<(), io::Error> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 && args.len() != 2 {
        panic!("incorrect number of arguments")
    }
    let blend_path = if args[1] == "--help" {
        println!("usage:");
        println!("{} --blend <blend file path>", args[0]);
        exit(-1);
    } else if args[1] == "--blend" {
        &args[2]
    } else {
        panic!("unknown arguments passed, see --help for more details")
    };
    print_blend(blend_path)?;
    Ok(())
}
