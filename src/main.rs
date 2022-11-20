use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];
    let redundant: u32 = args[2].parse().unwrap();

    let mut map = HashMap::new();
    let mut log1 = String::new();

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

        //TODO: 条件分岐の冗長性を減らす
        if response_time == "-" {
            if map.contains_key(ip_address) {
                let value: &mut (String, u32) = map.get_mut(ip_address).unwrap();
                value.1 += 1;
                if value.1 >= redundant {
                    log1.push_str(&format!("{} {} {}\n", ip_address, value.0, check_date));
                    map.remove(ip_address);
                }
            } else {
            //add data to map
            let key: String = ip_address.to_string();
            let value: (String, u32) = (check_date.to_string(), 1);
            map.entry(key).or_insert(value);
            }
        } else {
            if map.contains_key(ip_address) {
                let value: &mut (String, u32) = map.get_mut(ip_address).unwrap();
                if value.1 >= redundant {
                    log1.push_str(&format!("{} {} {}\n", ip_address, value.0, check_date));
                }
                map.remove(ip_address);
            }
        }
    }
    println!("{:?}", map);
    println!("{}", log1);
    //write to log1.txt
    log1_file.write_all(log1.as_bytes()).expect("ログファイル1に書き込めません");
}