use blend::Blend;
use libflate::gzip::Decoder;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    process::exit,
};

/// Checks if the file header contains the magic bytes to represent a
/// gzip file
///
/// From Blender's `BLI_file_magic_is_gzip()` in `fileops.c`
pub fn file_magic_is_gzip(data: &[u8]) -> bool {
    // GZIP itself starts with the magic bytes 0x1f 0x8b. The third
    // byte indicates the compression method, which is 0x08 for
    // DEFLATE.
    data[0] == 0x1f && data[1] == 0x8b && data[2] == 0x08
}

/// Checks if the file header contains the magic bytes to represent a
/// zstd file
///
/// From Blender's `BLI_file_magic_is_zstd()` in `fileops.c`
pub fn file_magic_is_zstd(data: &[u8]) -> bool {
    // ZSTD files consist of concatenated frames, each either a Zstd
    // frame or a skippable frame.  Both types of frames start with a
    // magic number: 0xFD2FB528 for Zstd frames and 0x184D2A5* for
    // skippable frames, with the * being anything from 0 to F.
    //
    // To check whether a file is Zstd-compressed, we just check
    // whether the first frame matches either. Seeking through the
    // file until a Zstd frame is found would make things more
    // complicated and the probability of a false positive is rather
    // low anyways.
    //
    // Note that LZ4 uses a compatible format, so even though its
    // compressed frames have a different magic number, a valid LZ4
    // file might also start with a skippable frame matching the
    // second check here.
    //
    // For more details, see
    // https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md

    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

    magic == 0xFD2FB528 || (magic >> 4) == 0x184D2A5
}

fn print_blend(blend_path: impl AsRef<Path>) -> Result<(), io::Error> {
    println!("{}", blend_path.as_ref().display());
    let mut file = File::open(blend_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let data = if data[0..7] != *b"BLENDER" {
        if file_magic_is_gzip(&data) {
            let mut decoder = Decoder::new(&data[..])?;
            let mut gzip_data = Vec::new();
            decoder.read_to_end(&mut gzip_data)?;

            gzip_data
        } else if file_magic_is_zstd(&data) {
            zstd::decode_all(std::io::Cursor::new(data))?
        } else {
            panic!("blend file compressed using unknown compression technique");
        }
    } else {
        data
    };

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
