mod bwtstring;
use bwtstring::*;

use std::io::{self, BufRead};

// Simple interactive testing code
fn main() {
    println!("Please enter bytes in decimal separated by whitespace or comma. Non-byte-parsable values will be ignored");
    for line in io::stdin().lock().lines().filter_map(|res| res.ok()) {
        // Split line by commas or spaces into different u8s
        let bytes = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .map(|str| str.parse::<u8>())
            .filter_map(|parse_result| parse_result.ok())
            .collect::<Vec<_>>();

        let bwt = BWTString::new(bytes).forward_transform();
        println!("Burrows-Wheeler transform: {bwt}");
    }
}
