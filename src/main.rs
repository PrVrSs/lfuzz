use std::collections::HashSet;
use libc::{c_uint, c_ulong};
use rand::Rng;
use clap::Parser;
use x11::keysym;
use lfuzz::app::App;
use lfuzz::statistics::Statistics;


fn black_list() -> HashSet<c_uint>{
    [
        keysym::XK_Super_L,
        keysym::XK_Win_L,
        keysym::XK_Win_R,
        keysym::XK_F1,
    ].iter().cloned().collect()
}


fn fuzz(app: App, blacklist: HashSet<c_uint>, pred: u32) {
    let mut rng = rand::thread_rng();
    let mut stats = Statistics::default();

    loop {
        let rand_value = rng.gen::<u16>() as u32;
        if blacklist.contains(&rand_value) {
            continue;
        }

        std::thread::sleep(std::time::Duration::new(0, 500));

        stats.count += 1;
        stats.unique_keys.insert(rand_value);

        app.press(rand_value as c_ulong);

        if stats.count == pred {
            break;
        }
    }

    println!("{:?}", stats);
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Target application
    #[clap(short, long)]
    target: String,

    /// Number of fuzz times
    #[clap(short, long, default_value_t = 100)]
    count: u32,
}


fn main() {
    let args = Args::parse();

    let app = App::attach(args.target).expect("Something went wrong");
    app.activate();

    let mut blacklist = black_list();

    for key in 0x0..=0x22 {
        blacklist.insert(key);
    }

    for key in 0x7f..=0xffff {
        blacklist.insert(key);
    }

    fuzz(app, blacklist, args.count);
}
