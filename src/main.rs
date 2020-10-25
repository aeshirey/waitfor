use clap::{App, Arg};
use std::time::Duration;

mod misc;
mod waitfor;
use waitfor::Wait;

fn main() -> Result<(), ()> {
    let matches = App::new("waitfor")
        .version("0.1")
        .author("Adam Shirey <adam@shirey.ch>")
        .about("")
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("interval")
                .help("The interval in seconds between condition checks")
                .required(false)
                .default_value("2")
                .takes_value(true)
                .validator(|a| {
                    if a.chars().all(|c| c.is_digit(10)) {
                        Ok(())
                    } else {
                        Err("Interval must be numeric".into())
                    }
                }),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("verbose")
                .required(false)
                .takes_value(false)
        )
        .arg(
            Arg::with_name("elapsed")
                .short("t")
                .long("elapsed")
                .value_name("duration-def")
                .help("Sleeps the specified time")
                .required(false)
                .takes_value(true)
                .validator(|a| {
                    misc::parse_duration(&a)
                        .map(|_| ())
                        .map_err(|_| format!("Invalid duration definition: {}", a))
                }),
        )
        .arg(
            Arg::with_name("exists")
                .short("e")
                .long("exists")
                .value_name("file-or-dir")
                .help("Delays until the specified file exists.")
                //.long_help(r#""#)
                .required(false)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("not-exists")
                .short("E")
                .long("not-exists")
                .value_name("file-or-dir")
                .help("Delays until the specified file no longer exists.")
                //.long_help(r#""#)
                .required(false)
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("get")
                .short("g")
                .long("get")
                .value_name("get")
                .help("Delays until an HTTP GET against the specified URL returns 200 OK or the passed status.")
                //.long_help(r#""#)
                .required(false)
                .multiple(true)
                .takes_value(true)
                .validator(|a|
                    misc::parse_http_get(&a)
                        .map(|_| ())
                        .map_err(|_| format!("Invalid HTTP GET definition: {}",a))
                )
        )
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let mut waitfors = Vec::new();

    let verbose : bool = matches.is_present("verbose");

    if let Some(elapsed) = matches.value_of("elapsed") {
        let duration = misc::parse_duration(elapsed).unwrap();

        waitfors.push(Wait::Elapsed {
            end_instant: std::time::Instant::now().checked_add(duration).unwrap(),
        });
    }

    if let Some(pids) = matches.values_of("pid") {
        for pid in pids {
            waitfors.push(Wait::Pid(pid.parse().unwrap()));
        }
    }

    if let Some(paths) = matches.values_of("exists") {
        for path in paths {
            waitfors.push(Wait::Exists {
                not: false,
                path: path.into(),
            });
        }
    }

    if let Some(paths) = matches.values_of("not-exists") {
        for path in paths {
            waitfors.push(Wait::Exists {
                not: true,
                path: path.into(),
            });
        }
    }

    if let Some(urlargs) = matches.values_of("get") {
        for urlarg in urlargs {
            let (status, url) = misc::parse_http_get(urlarg).unwrap();
            waitfors.push(Wait::HttpGet { url, status });
        }
    }

    if waitfors.is_empty() {
        return Err(());
    }

    let process_started = std::time::Instant::now();

    let interval = Duration::from_secs_f64(matches.value_of("interval").unwrap().parse().unwrap());

    loop {
        let start = std::time::Instant::now();
        for waitfor in waitfors.iter() {
                if verbose {
                    println!("Checking {:?}", waitfor);
                }

            if waitfor.condition_met() {
                if verbose {
                    println!("Waited {}", process_started.elapsed().as_secs());
                }
                return Ok(());
            }
        }
        let loop_time = start.elapsed();
        if interval > loop_time {
            std::thread::sleep(interval - loop_time);
        }
    }
}