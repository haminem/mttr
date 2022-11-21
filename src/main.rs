use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::Ipv4Addr;

fn get_subnet(ip_address: &str) -> String {
    let data: Vec<&str> = ip_address.split("/").collect::<Vec<&str>>();
    let ip: Ipv4Addr = data[0].parse().unwrap();
    let mask: u32 = data[1].parse().unwrap();
    let subnet: Ipv4Addr = masker(ip, mask);
    format!("{}/{}", subnet, mask)
}

fn masker(ip: Ipv4Addr, mask: u32) -> Ipv4Addr {
    let ip: u32 = ip.into();
    let mask: u32 = !0 << (32 - mask);
    let subnet: u32 = ip & mask;
    subnet.into()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename: &String = &args[1];
    let redundant: u32 = args[2].parse().unwrap();
    let response_time_average_range: u32 = args[3].parse().unwrap();
    let response_time_average_capacity: u32 = args[4].parse().unwrap();

    let mut map_timeout: HashMap<String, (String, u32)> = HashMap::new();
    let mut map_response_time: HashMap<String, (String, u32, VecDeque<u32>, bool, String)> =
        HashMap::new();
    let mut map_subnet: HashMap<String, (HashMap<String, bool>, String, String)> = HashMap::new();
    let mut log1: String = String::new();
    let mut log2: String = String::new();
    let mut log3: String = String::new();
    let mut log1_file: File = File::create("log1.txt").expect("ログファイル１が作成できません");
    let mut log2_file: File = File::create("log2.txt").expect("ログファイル２が作成できません");
    let mut log3_file: File = File::create("log3.txt").expect("ログファイル３が作成できません");

    let f: File = File::open(filename).expect("ファイルが見つかりません");
    let reader: BufReader<File> = BufReader::new(f);
    for line in reader.lines() {
        let line: String = line.unwrap();
        let data: Vec<&str> = line.split(",").collect();
        let ip_address: &str = data[1];
        map_subnet
            .entry(get_subnet(ip_address))
            .or_insert((HashMap::new(), String::new(), String::new()))
            .0
            .insert(ip_address.to_string(), false);
    }

    let f: File = File::open(filename).expect("ファイルが見つかりません");
    let reader: BufReader<File> = BufReader::new(f);
    for line in reader.lines() {
        let line: String = line.unwrap();
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
            map_subnet
                .get_mut(&get_subnet(ip_address))
                .unwrap()
                .0
                .insert(ip_address.to_string(), true);
        } else {
            map_timeout
                .entry(ip_address.to_string())
                .and_modify(|value: &mut (String, u32)| {
                    if value.1 >= redundant {
                        log1.push_str(&format!("{} {}~{}\n", ip_address, value.0, check_date));
                    }
                    value.1 = 0;
                });
            map_response_time
                .entry(ip_address.to_string())
                .and_modify(|value: &mut (String, u32, VecDeque<u32>, bool, String)| {
                    value.1 += 1;
                    value.2.push_back(response_time.parse().unwrap());
                    if value.1 >= response_time_average_range {
                        if value.1 > response_time_average_range {
                            value.2.pop_front();
                        }
                        let sum: u32 = value.2.iter().sum();
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
                    }
                })
                .or_insert((
                    check_date.to_string(),
                    1,
                    VecDeque::from([response_time.parse().unwrap()]),
                    false,
                    check_date.to_string(),
                ));
            map_subnet
                .get_mut(&get_subnet(ip_address))
                .unwrap()
                .0
                .insert(ip_address.to_string(), false);
        }
        for (subnet, value) in map_subnet.iter_mut() {
            let mut is_all_failure: bool = true;
            for (_ip, status) in value.0.iter() {
                if !*status {
                    is_all_failure = false;
                    break;
                }
            }
            if is_all_failure {
                if value.1 == "" {
                    value.1 = check_date.to_string();
                }
                value.2 = check_date.to_string();
            } else {
                if value.1 != "" {
                    log3.push_str(&format!("{} {}~{}\n", subnet, value.1, value.2));
                    value.1 = String::new();
                    value.2 = String::new();
                }
            }
        }
    }
    map_response_time.iter().for_each(|(key, value)| {
        if value.3 {
            log2.push_str(&format!("{} {}~{}\n", key, value.0, value.4));
        }
    });
    
    println!("{:?}\n", map_subnet);
    println!("{:?}\n", map_timeout);
    println!("{}", log1);
    println!("{}", log2);
    println!("{}", log3);
    //write to log1.txt
    log1_file
        .write_all(log1.as_bytes())
        .expect("ログファイル1に書き込めません");
    //write to log2.txt
    log2_file
        .write_all(log2.as_bytes())
        .expect("ログファイル2に書き込めません");
    //write to log3.txt
    log3_file
        .write_all(log3.as_bytes())
        .expect("ログファイル3に書き込めません");
}
