extern crate argparse;
extern crate png_decoder;

use png_decoder::raw::RawChunk;

use argparse::{ArgumentParser, Store};

struct Arguments {
    file: String,
}


fn parse_args() -> Arguments {
    let mut file = String::new();

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Parse a PNG header.");
        ap.refer(&mut file)
            .add_option(&["-f", "--file"], Store,
                "PNG file to parse.")
            .required();
        ap.parse_args_or_exit();
    }

    Arguments {
        file: file,
    }
}

fn main() {
    let args = parse_args();

    for chunk in png_decoder::raw::read_png_raw_from_file(&args.file).ok().expect("Failed to parse png!") {
        println!("Chunk type: {}, Chunk length: {}", std::str::from_utf8(&chunk.chunk_type()).ok().expect("Not utf8?!"), chunk.chunk_data().len());
    }
}
