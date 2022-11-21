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

    struct Timeout {
        check_date: String,
        range: u32,
    }
    struct Overload {
        start_date: String,
        range: u32,
        queue: VecDeque<u32>,
        is_timeout: bool,
        previous_date: String,
    }
    struct Down {
        map_ip: HashMap<String, bool>,
        start_date: String,
        check_date: String,
    }
    //map_timeout is HashMap<ip, (check_date, range)>
    let mut map_timeout: HashMap<String, Timeout> = HashMap::new();
    //map_overload is HashMap<ip, (check_date, range, queue, is_timeout, previous_date)>
    let mut map_overload: HashMap<String, Overload> = HashMap::new();
    //map_subnet is HashMap<subnet, (map_ip, start_date, check_date)>
    let mut map_subnet: HashMap<String, Down> = HashMap::new();

    create_dir("result").unwrap_or_default();
    let mut ttr_server_file: File =
        File::create("result/ttr_server.txt").expect("Unable to create file ttr_server.txt");
    let mut overload_file: File =
        File::create("result/overload.txt").expect("Unable to create file overload.txt");
    let mut tty_subnet_file: File =
        File::create("result/tty_subnet.txt").expect("Unable to create file tty_subnet.txt");

    let mut ttr_server_log: String = String::new();
    let mut overload_log: String = String::new();
    let mut tty_subnet_log: String = String::new();

    let f: File = File::open(filename).expect("Unable to open file");
    let reader: BufReader<File> = BufReader::new(f);
    for line in reader.lines() {
        let line: String = line.unwrap();
        let data: Vec<&str> = line.split(",").collect();
        let ip_address: &str = data[1];
        map_subnet
            .entry(get_subnet(ip_address))
            .or_insert(Down {
                map_ip: HashMap::new(),
                start_date: String::new(),
                check_date: String::new(),
            })
            .map_ip
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
                .and_modify(|timeout: &mut Timeout| {
                    timeout.range += 1;
                })
                .or_insert(Timeout {
                    check_date: check_date.to_string(),
                    range: 1,
                });
            map_overload.remove(ip_address);
            map_subnet
                .get_mut(&get_subnet(ip_address))
                .unwrap()
                .map_ip
                .insert(ip_address.to_string(), true);
        } else {
            map_timeout
                .entry(ip_address.to_string())
                .and_modify(|timeout: &mut Timeout| {
                    if timeout.range >= redundant {
                        ttr_server_log.push_str(&format!(
                            "{} {} ~ {}\n",
                            ip_address,
                            formatter(&timeout.check_date),
                            formatter(&check_date.to_string())
                        ));
                    }
                    timeout.range = 0;
                });
            map_overload
                .entry(ip_address.to_string())
                .and_modify(|overload: &mut Overload| {
                    overload.range += 1;
                    overload.queue.push_back(response_time.parse().unwrap());
                    if overload.range >= response_time_average_range {
                        if overload.range > response_time_average_range {
                            overload.queue.pop_front();
                        }
                        let sum: u32 = overload.queue.iter().sum();
                        let average: u32 = sum / response_time_average_range;
                        if average >= response_time_average_capacity {
                            if !overload.is_timeout {
                                overload.start_date = check_date.to_string();
                            }
                            overload.is_timeout = true;
                        }
                        if average < response_time_average_capacity && overload.is_timeout {
                            overload_log.push_str(&format!(
                                "{} {} ~ {}\n",
                                ip_address,
                                formatter(&overload.start_date),
                                formatter(&overload.previous_date)
                            ));
                            overload.is_timeout = false;
                        }
                        overload.previous_date = check_date.to_string();
                    }
                })
                .or_insert(Overload {
                    start_date: check_date.to_string(),
                    range: 1,
                    queue: VecDeque::new(),
                    is_timeout: false,
                    previous_date: String::new(),
                });
            map_subnet
                .get_mut(&get_subnet(ip_address))
                .unwrap()
                .map_ip
                .insert(ip_address.to_string(), false);
        }
        //check subnet timeout
        for (subnet, down) in map_subnet.iter_mut() {
            let mut is_all_failure: bool = true;
            for (_ip, status) in down.map_ip.iter() {
                if !*status {
                    is_all_failure = false;
                    break;
                }
            }
            if is_all_failure {
                if down.start_date == "" {
                    down.start_date = check_date.to_string();
                }
                down.check_date = check_date.to_string();
            } else {
                if down.start_date != "" {
                    tty_subnet_log.push_str(&format!(
                        "{} {} ~ {}\n",
                        subnet,
                        formatter(&down.start_date),
                        formatter(&down.check_date)
                    ));
                    down.start_date = String::new();
                    down.check_date = String::new();
                }
            }
        }
        //response_time_average_range is 1
        if response_time_average_range == 1 {
            map_overload
                .entry(ip_address.to_string())
                .and_modify(|overload: &mut Overload| {
                    if response_time.parse::<u32>().unwrap() >= response_time_average_capacity {
                        if !overload.is_timeout {
                            overload.start_date = check_date.to_string();
                        }
                        overload.is_timeout = true;
                    }
                });
        }
    }
    //end range timeout of overload
    map_overload.iter().for_each(|(ip_address, overload)| {
        if overload.is_timeout {
            overload_log.push_str(&format!(
                "{} {} ~ {}\n",
                ip_address,
                formatter(&overload.start_date),
                formatter(&overload.previous_date)
            ));
        }
    });
    ttr_server_file
        .write_all(ttr_server_log.as_bytes())
        .expect("Unable to write data to ttr_server.txt");
    overload_file
        .write_all(overload_log.as_bytes())
        .expect("Unable to write data to overload.txt");
    tty_subnet_file
        .write_all(tty_subnet_log.as_bytes())
        .expect("Unable to write data to tty_subnet.txt");
}
