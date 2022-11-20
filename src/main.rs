use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];

    let mut map = HashMap::new();
    let mut log = String::new();

    let f = File::open(filename).expect("ファイルが見つかりません");
    let reader = BufReader::new(f);

    let mut log1_file = File::create("log1.txt").expect("ログファイル１が作成できません");

    for line in reader.lines() {
        let line: String = line.unwrap();
        // data is [check_date, ip_address, response_time]
        let data: Vec<&str> = line.split(",").collect();
        let check_date: &str = data[0];
        let ip_address: &str = data[1];
        let response_time: &str = data[2];
        println!("{} {} {}", check_date, ip_address, response_time);

        if response_time == "-" {
            //add data to map
            let key: String = ip_address.to_string();
            let value: String = check_date.to_string();
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
    //write to log1.txt
    log1_file.write_all(log.as_bytes()).expect("ファイルに書き込めません");
}