use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let f = File::open(filename).expect("ファイルが見つかりません");
    let reader = BufReader::new(f);
    for line in reader.lines() {
        let line: String = line.unwrap();
        let data: Vec<&str> = line.split(',').collect();
        println!("{} {} {}", data[0], data[1], data[2]);
    }
}
