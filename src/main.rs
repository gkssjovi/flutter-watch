#[macro_use]
extern crate clap;

use clap::{App, ArgMatches};
use colour::{green_ln, red_ln, yellow_ln};
use nix::sys::signal::kill;
use nix::sys::signal::Signal::SIGUSR1;
use nix::unistd::Pid;
use notify::{raw_watcher, watcher, DebouncedEvent, RawEvent, RecursiveMode, Watcher};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    let mut flutter_watch = FlutterWatch::new(Config::new());
    flutter_watch.start();
}

enum WatchEvent {
    DebouncedEvent,
    RawEvent,
}

pub struct Config {
    watch_event: WatchEvent,
}

impl Config {
    pub fn new() -> Self {
        Self {
            watch_event: WatchEvent::RawEvent,
        }
    }
}

pub struct FlutterWatch {
    pub config: Config,
    pid: i32,
    target: String,
}

impl FlutterWatch {
    pub fn new(config: Config) -> Self {
        Self {
            config: config,
            pid: 0,
            target: String::from(""),
        }
    }

    pub fn start(&mut self) {
        let yaml = load_yaml!("cli.yml");
        let matches = App::from_yaml(yaml).get_matches();

        
        let default_event_type = if let WatchEvent::RawEvent = self.config.watch_event { "raw" } else { "debounce" };
        let event_type = matches.value_of("event-type").unwrap_or(default_event_type);

        self.config.watch_event = match event_type {
            "raw" => WatchEvent::RawEvent,
            "debounce" => WatchEvent::DebouncedEvent,
            _ => WatchEvent::RawEvent,
        };

        self.pid = self.get_pid(&matches);
        self.target = self.get_target(&matches);

        green_ln!("Watching -> {}", self.target);

        match self.config.watch_event {
            WatchEvent::DebouncedEvent => self.start_disptach_event(matches),
            WatchEvent::RawEvent => self.start_raw_event(matches),
        }
    }

    fn start_disptach_event(&mut self, matches: ArgMatches) {
        let (sender, receiver) = channel();
        let mut watcher = watcher(sender, Duration::from_secs(1)).unwrap();

        watcher
            .watch(&self.target, RecursiveMode::Recursive)
            .unwrap();

        loop {
            match receiver.recv() {
                Ok(event) => match event {
                    DebouncedEvent::NoticeWrite { .. } => {}
                    DebouncedEvent::NoticeRemove { .. } => {}
                    _ => match kill(Pid::from_raw(self.pid), SIGUSR1) {
                        Ok(result) => {
                            green_ln!("{:?} ", event);
                            result
                        }
                        Err(error) => {
                            red_ln!("[{}] PID not found. Probably the flutter run is not started ({:?}).", self.pid, error);
                            self.pid = self.get_pid(&matches);
                        }
                    },
                },
                Err(e) => {
                    red_ln!("watch error: {:?}", e);
                }
            }
        }
    }

    fn start_raw_event(&mut self, matches: ArgMatches) {
        let (sender, receiver) = channel();
        let mut watcher = raw_watcher(sender).unwrap();

        watcher
            .watch(&self.target, RecursiveMode::Recursive)
            .unwrap();

        loop {
            match receiver.recv() {
                Ok(RawEvent {
                    path: Some(path),
                    op: Ok(op),
                    cookie,
                }) => {
                    match kill(Pid::from_raw(self.pid), SIGUSR1) {
                        Ok(result) => {
                            green_ln!("[{:?}] {:?} ({:?})", op, path, cookie);
                            result
                        }
                        Err(error) => {
                            red_ln!("[{}] PID not found. Probably the flutter run is not started ({:?}).", self.pid, error);
                            self.pid = self.get_pid(&matches);
                        }
                    }
                }
                Ok(event) => {
                    yellow_ln!("broken event: {:?}", event);
                }
                Err(e) => {
                    red_ln!("watch error: {:?}", e);
                }
            }
        }
    }

    fn get_target(&self, matches: &ArgMatches) -> String {
        let current_dir_str = current_dir().unwrap();
        let current_path = Path::new(&current_dir_str).join("lib");
        let target = matches
            .value_of("TARGET")
            .unwrap_or(&current_path.to_str().unwrap());

        if !Path::new(&target).exists() {
            red_ln!("The target file does not exits ({})", target);
            std::process::exit(1);
        }

        target.to_string()
    }

    fn get_pid(&self, matches: &ArgMatches) -> i32 {
        let pid: i32;
        let pid_number = matches
            .value_of("pid")
            .unwrap_or("-1")
            .parse::<i32>()
            .unwrap();

        if pid_number > -1 {
            pid = pid_number;
        } else {
            let pid_file = matches.value_of("pid-file").unwrap_or("/tmp/flutter.pid");

            if !Path::new(&pid_file).exists() {
                red_ln!("The pid file does not exits ({})", pid_file);
                std::process::exit(1);
            }

            let mut contents = String::new();
            let mut file = File::open(&pid_file).unwrap();
            file.read_to_string(&mut contents).unwrap();
            pid = contents.parse::<i32>().unwrap();
        }

        pid
    }
}
