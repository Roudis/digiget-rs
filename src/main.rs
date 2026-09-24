//! Display digimon sprites in your terminal.

use clap::Parser;
use digiget::cli::Args;
use digiget::digimon::Digimon;
use digiget::list::List;
use digiget::sprites;
use std::process::exit;

fn main() {
    let list = List::read();
    let args = Args::parse();

    if args.list {
        for (id, name, file) in list.entries() {
            println!("{id:>4}  {name:<40} {file}");
        }
        return;
    }

    if args.digimon.is_empty() {
        eprintln!("you must specify the digimon you want to display");
        exit(1);
    }

    let digimons: Vec<Digimon> = args
        .digimon
        .into_iter()
        .map(|x| Digimon::new(x, &list))
        .collect();

    let combined = sprites::combine(&digimons);
    let combined = match (args.scale, args.no_fit) {
        (Some(factor), _) => sprites::scale(&combined, factor),
        (None, true) => combined,
        (None, false) => match sprites::terminal_size() {
            Some((cols, rows)) => sprites::fit(&combined, cols, rows),
            None => combined,
        },
    };
    if !args.hide_name {
        let names: Vec<&str> = digimons.iter().map(|x| x.name.as_ref()).collect();
        eprintln!("{}", names.join(", "));
    }

    println!("{}", showie::render(&combined));
}
