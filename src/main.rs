use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut map = HashMap::new();
    let mut log = String::new();

    let f = File::open(filename).expect("ファイルが見つかりません");
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line: String = line.unwrap();
        // data is [check_date, ip_address, response_time]
        let data: Vec<&str> = line.split(",").collect();
        let check_date = data[0];
        let ip_address = data[1];
        let response_time = data[2];
        println!("{} {} {}", check_date, ip_address, response_time);

        if response_time == "-" {
            //add data to map
            let key = ip_address.to_string();
            let value = check_date.to_string();
            map.entry(key).or_insert(value);
        } else {
            for (key, value) in &map {
                if key == ip_address {
                    log.push_str(&format!("{} {}~{}\n", ip_address, value, check_date));
                }
            }
            //remove data from map
            map.remove(ip_address);
        }
    }
    println!("{:?}", map);
    println!("{}", log);
}