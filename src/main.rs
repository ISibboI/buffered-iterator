use crate::buffered_parser::BufferedParser;

mod buffered_parser;
mod buffered_parser_ref;

fn main() {
    let buffer = [1, 5, 0, 4, 3, 4, 5, 6];
    let parser = BufferedParser::new(buffer.as_slice());

    for slice in parser {
        println!("slice: {slice:?}");
    }
}
