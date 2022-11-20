use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];
    let redundant: u32 = args[2].parse().unwrap();
    let response_time_average_range: u32 = args[3].parse().unwrap();
    let response_time_average_capacity: u32 = args[4].parse().unwrap();

    let mut map_timeout: HashMap<String, (String, u32)> = HashMap::new();
    let mut map_response_time: HashMap<String, (String, u32, VecDeque<u32>, bool, String)> = HashMap::new();

    let mut log1: String = String::new();
    let mut log2: String = String::new();

    let f: File = File::open(filename).expect("ファイルが見つかりません");
    let reader: BufReader<File> = BufReader::new(f);

    let mut log1_file: File = File::create("log1.txt").expect("ログファイル１が作成できません");
    let mut log2_file: File = File::create("log2.txt").expect("ログファイル２が作成できません");

    for line in reader.lines() {
        let line: String = line.unwrap();
        // data is [check_date, ip_address, response_time]
        let data: Vec<&str> = line.split(",").collect();
        let check_date: &str = data[0];
        let ip_address: &str = data[1];
        let response_time: &str = data[2];
        println!("{} {} {}", check_date, ip_address, response_time);
        if response_time == "-" {
            map_timeout
                .entry(ip_address.to_string())
                .and_modify(|value: &mut (String, u32)| {
                    value.1 += 1;
                })
                .or_insert((check_date.to_string(), 1));
            map_response_time.remove(ip_address);
        } else {
            map_timeout
                .entry(ip_address.to_string())
                .and_modify(|value: &mut (String, u32)| {
                    if value.1 >= redundant {
                        log1.push_str(&format!("{} {} {}\n", ip_address, value.0, check_date));
                    }
                    value.1 = 0;
                });
            //TODO: response_time_average_rangeが1の場合の処理
            map_response_time
            .entry(ip_address.to_string())
                .and_modify(|value: &mut (String, u32, VecDeque<u32>, bool, String)| {
                value.1 += 1;
                //TODO: 冗長さの修正
                value.2.push_back(response_time.parse().unwrap());
                if value.1 >= response_time_average_range {
                    if value.1 > response_time_average_range {
                        value.2.pop_front();
                    }
                    let sum: u32 = value.2.iter().sum();
                    println!("sum: {}", sum);
                    let average: u32 = sum / response_time_average_range;
                    if average >= response_time_average_capacity {
                        if !value.3 {
                            value.0 = value.4.clone();
                        }
                        value.3 = true;
                    }
                    if average < response_time_average_capacity && value.3 {
                        log2.push_str(&format!("{} {}~{}\n", ip_address, value.0, value.4));
                        value.3 = false;
                    }
                    value.4 = check_date.to_string();
                    println!("bool {}", value.3);
                }
            })
                .or_insert((check_date.to_string(), 1, VecDeque::from([response_time.parse().unwrap()]), false, check_date.to_string()));
        }
    }
    map_response_time
        .iter()
        .for_each(|(key, value)| {
            if value.3 {
                log2.push_str(&format!("{} {}~{}\n", key, value.0, value.4));
            }
        });
    println!("{:?}", map_timeout);
    println!("\n");
    println!("{}", log1);
    println!("{}", log2);
    //write to log1.txt
    log1_file
        .write_all(log1.as_bytes())
        .expect("ログファイル1に書き込めません");
    //write to log2.txt
    log2_file
        .write_all(log2.as_bytes())
        .expect("ログファイル2に書き込めません");
}
