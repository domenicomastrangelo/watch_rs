# watch_rs
Classic linux "watch" command rewritten in rust.

run with
```
watch -n 1 -d <COMMAND>
```

Usage
 
```
Usage: watch [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to run

Options:
  -n, --interval <NUMBER>  Interval at which the executable is run
  -d, --difference         Highlight the difference between refreshes
  -h, --help               Print help
  -V, --version            Print version
```

![image](https://github.com/domenicomastrangelo/watch_rs/assets/7526063/42734095-8af9-4ca0-8eec-06c89be778de)
