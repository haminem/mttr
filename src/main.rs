use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::{create_dir, File};
use std::io::{BufRead, BufReader, Write};
use std::net::Ipv4Addr;
use std::process::exit;

// date is a string in the format "YYYYMMDDHHMMSS"
fn formatter(date: &String) -> String {
    let year = &date[0..4];
    let month = &date[4..6];
    let day = &date[6..8];
    let hour = &date[8..10];
    let minute = &date[10..12];
    let second = &date[12..14];
    format!("{}/{}/{} {}:{}:{}", year, month, day, hour, minute, second)
}

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
    let filename: &String = &("test/".to_string() + &args[1]);
    let redundant: u32 = args[2].parse().unwrap();
    let response_time_average_range: u32 = args[3].parse().unwrap();
    let response_time_average_capacity: u32 = args[4].parse().unwrap();

    //validates the input
    if !filename.ends_with(".txt") {
        println!("The filename must be a .txt file");
        exit(1);
    }
    if redundant < 1 {
        println!("The redundant value must be greater than 0");
        exit(1);
    }
    if response_time_average_range < 1 {
        println!("The response_time_average_range value must be greater than 0");
        exit(1);
    }

    //map_timeout is HashMap<ip, (check_date, timeout)>
    let mut map_timeout: HashMap<String, (String, u32)> = HashMap::new();
    //map_response_time is HashMap<ip, (check_date, response_time_average, response_time_average_queue, is_timeout, pre_date)>
    let mut map_response_time: HashMap<String, (String, u32, VecDeque<u32>, bool, String)> =
        HashMap::new();
    //map_subnet is HashMap<subnet, (map_ip, subnet_timeout_reason, subnet_timeout_reason)>
    let mut map_subnet: HashMap<String, (HashMap<String, bool>, String, String)> = HashMap::new();

    create_dir("result").unwrap_or_default();
    let mut log1_file: File =
    File::create("result/log1.txt").expect("Unable to create file log1.txt");
    let mut log2_file: File =
    File::create("result/log2.txt").expect("Unable to create file log2.txt");
    let mut log3_file: File =
    File::create("result/log3.txt").expect("Unable to create file log3.txt");
    let mut log1: String = String::new();
    let mut log2: String = String::new();
    let mut log3: String = String::new();

    let f: File = File::open(filename).expect("Unable to open file");
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

    let f: File = File::open(filename).expect("Unable to open file");
    let reader: BufReader<File> = BufReader::new(f);
    for line in reader.lines() {
        let line: String = line.unwrap();
        let data: Vec<&str> = line.split(",").collect();
        let check_date: &str = data[0];
        let ip_address: &str = data[1];
        let response_time: &str = data[2];
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
                        log1.push_str(&format!(
                            "{} {} ~ {}\n",
                            ip_address,
                            formatter(&value.0),
                            formatter(&check_date.to_string())
                        ));
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
                                value.0 = check_date.to_string();
                            }
                            value.3 = true;
                        }
                        if average < response_time_average_capacity && value.3 {
                            log2.push_str(&format!(
                                "{} {} ~ {}\n",
                                ip_address,
                                formatter(&value.0),
                                formatter(&value.4)
                            ));
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
        //check subnet timeout
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
                    log3.push_str(&format!(
                        "{} {} ~ {}\n",
                        subnet,
                        formatter(&value.1),
                        formatter(&value.2)
                    ));
                    value.1 = String::new();
                    value.2 = String::new();
                }
            }
        }
        //response_time_average_range is 1
        if response_time_average_range == 1 {
            map_response_time.entry(ip_address.to_string()).and_modify(
                |value: &mut (String, u32, VecDeque<u32>, bool, String)| {
                    if response_time.parse::<u32>().unwrap() >= response_time_average_capacity {
                        if !value.3 {
                            value.0 = check_date.to_string();
                        }
                        value.3 = true;
                    }
                },
            );
        }
    }
    //end term timeout of response_time
    map_response_time.iter().for_each(|(key, value)| {
        if value.3 {
            log2.push_str(&format!(
                "{} {} ~ {}\n",
                key,
                formatter(&value.0),
                formatter(&value.4)
            ));
        }
    });
    log1_file
        .write_all(log1.as_bytes())
        .expect("Unable to write data to log1.txt");
    log2_file
        .write_all(log2.as_bytes())
        .expect("Unable to write data to log2.txt");
    log3_file
        .write_all(log3.as_bytes())
        .expect("Unable to write data to log3.txt");
}
