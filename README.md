# Description 

Flutter watch for files changes and hot reload directly from terminal 

# Installation

```sh
git clone https://github.com/gkssjovi/flutter-watch.git
cd flutter-watch
cargo build --release
ln -s $PWD/target/release/flutter-watch /usr/local/bin/flutter-watch
```

Add to your `~/.zshrc` file
```sh
alias flutter-run="flutter run --pid-file /tmp/flutter.pid"
```

# Usage

Use a different terminal window 

`flutter-run` or `flutter run --pid-file /tmp/flutter.pid`

If you are outside of the flutter project directory
```sh
flutter-watch /path/to/your/flutter/project
```

If you are in the flutter project directory
```sh
flutter-watch
``` 

# Help

```sh
flutter-watch -h

USAGE:
    flutter-watch [OPTIONS] [TARGET]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --event-type <EVENT_TYPE>    'raw' or 'debounce'. Example (--event-type debounce)
    -p, --pid <PID>                  Using a pid. Example (--pid 12345)
    -f, --pid-file <FILE>            Sets the pid file location. Example: (--pid-file /tmp/flutter.pid)

ARGS:
    <TARGET>    Set the project target. Example (flutter-watch /path/to/your/flutter/project/lib)
```

