mod client;
mod server;

use getopts::Options;
use std::env;
use std::io::{stdout, Write};

fn main() {
    if cfg!(debug_assertions) {
        println!("!! WARNING: You are running a not optimized version of nettest !!");
        println!("!! Please use the --release build switch for any serious tests !!");
        println!();
    }

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();

    opts.optflag("h", "help", "Shows this text");
    opts.optflag("s", "server", "Launches a server");
    opts.optflag("m", "mac", "Whether to MAC packets or not at the server");
    opts.optopt("c", "client", "connects to a server", "SERVER_IP");
    opts.optopt(
        "t",
        "time",
        "time to test for in seconds (default: 10)",
        "TIME",
    );
    opts.optopt(
        "p",
        "port",
        "the port to listen on and connect to (default: 5001)",
        "PORT",
    );
    opts.optopt(
        "b",
        "bind",
        "Server bind address (default: \"0.0.0.0\")",
        "ADDR",
    );
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            // Unwrap is fine here because writing to stderr shouldn't fail
            writeln!(&mut std::io::stderr(), "{}", f.to_string()).unwrap();
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let mut has_done_stuff = false;

    if matches.opt_present("s") {
        has_done_stuff = true;
        let port = matches
            .opt_str("p")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(5001);
        let bind = matches
            .opt_str("b")
            .unwrap_or_else(|| "0.0.0.0".to_string());
        if matches.opt_present("m") {
            launch_server(port, &bind, true);
        } else {
            launch_server(port, &bind, false);
        }
    }

    if matches.opt_present("c") {
        has_done_stuff = true;
        let host = &matches.opt_str("c").unwrap();
        let port = matches
            .opt_str("p")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(5001);
        let time = matches
            .opt_str("t")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(10u64);
        match run_client(host, port, time) {
            Ok(_) => {}
            Err(x) => {
                writeln!(&mut std::io::stderr(), "Error during test: {}", x)
                    .unwrap()
            }
        };
    }

    if !has_done_stuff {
        print_usage(&program, opts);
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run_client(
    host: &str,
    port: u16,
    time: u64,
) -> Result<(), ::std::io::Error> {
    let mut c = client::TestClient::new(host, port)?;

    /*
    print!("Testing ping... ");
    stdout().flush()?;
    println!("done, {:.*} ms", 2, c.test_ping(20)?);

    print!("Testing download... ");
    stdout().flush()?;
    println!(
        "done, {}",
        format_speed(c.test_downstream(time * 1_000u64)?)
    );
    */

    print!("Testing upload... ");
    stdout().flush()?;
    println!("done, {}", format_speed(c.test_upstream(time * 1_000u64)?));

    Ok(())
}

fn launch_server(port: u16, listen: &str, should_mac: bool) {
    println!("Listening with should_mac={}...", should_mac);
    let s = server::TestServer::new(port, listen, should_mac);
    s.listen();
}

fn format_speed(speed: f64) -> String {
    let mut speed = speed;
    let units = ["bit/s", "kbit/s", "Mbit/s", "Gbit/s", "Tbit/s?!"];
    let mut idx = 0;
    while speed > 1024f64 && idx < 4 {
        idx += 1;
        speed /= 1024f64;
    }

    format!("{:.3} {}", speed, units[idx])
}
