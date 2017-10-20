extern crate clap;
extern crate dent;
extern crate term_size;

use clap::{App, Arg};
use dent::plot;
use dent::summary::Summary;
use dent::t_test::{SigLevel, TTest, welch_t_test};

use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead, BufReader};


fn print_summary(s: &Summary) {
    println!("N\tMin\tMax\tMedian\tMean\tStdDev\tStdErr");
    println!(
        "{}\t{:0.2}\t{:0.2}\t{:0.2}\t{:0.2}\t{:0.2}\t{:0.2}",
        s.size(),
        s.min(),
        s.max(),
        s.median(),
        s.mean(),
        s.standard_deviation(),
        s.standard_error(),
    );
}

fn print_t_test(t_test: &TTest) {
    println!("T\tDF\tAlpha\tCrit\tRejectNull");
    println!(
        "{:0.3}\t{}\t{:0.3}\t{:0.3}\t{}",
        t_test.t,
        t_test.df,
        t_test.alpha,
        t_test.crit,
        t_test.reject,
    );
}

fn summarize_file(path: &str, lax_parsing: bool) -> Summary {
    let p = Path::new(path);
    let f = File::open(p).unwrap();
    let reader = BufReader::new(f);

    let data = read_data(reader, lax_parsing);

    Summary::new(&data).unwrap()
}

fn read_data<R>(reader: R, lax_parsing: bool) -> Vec<f64> where R: BufRead {
    let mut data: Vec<f64> = vec![];

    for l in reader.lines() {
        let s = l.unwrap().trim().to_string();

        if s.is_empty() {
            continue;
        }

        match s.parse() {
            Ok(d) => data.push(d),
            err => if !lax_parsing { err.unwrap(); }
        }
    }

    data
}

fn parse_alpha(arg: &str) -> SigLevel {
    match arg {
        ".001" => SigLevel::Alpha001,
        ".005" => SigLevel::Alpha005,
        ".01"  => SigLevel::Alpha010,
        ".025" => SigLevel::Alpha025,
        ".05"  => SigLevel::Alpha050,
        ".1"   => SigLevel::Alpha100,
        _ => panic!(),
    }
}

fn summarize_stdin(lax_parsing: bool) -> Summary {
    let stdin = io::stdin();
    let data = read_data(stdin.lock(), lax_parsing);

    Summary::new(&data).unwrap()
}

fn display_t_test(
    summary1: &Summary,
    summary2: &Summary,
    alpha: SigLevel,
    draw_plot: bool,
    width: usize,
    ascii: bool,
) {
    let t_test = welch_t_test(&summary1, &summary2, alpha);

    if draw_plot {
        let p = plot::comparison_plot(&[&summary1, &summary2], width, ascii, true);
        println!("{}\n", p);
    }

    print_summary(&summary1);
    println!();
    print_summary(&summary2);
    println!();
    print_t_test(&t_test);
}

fn main() {
    let matches = App::new("dent")
        .version("0.3.0")
        .author("Joe Ranweiler <joe@lemma.co>")
        .about("A tiny tool for t-tests &c.")
        .arg(Arg::with_name("stdin")
             .short("s")
             .long("stdin")
             .help("Read and summarize data from stdin"))
        .arg(Arg::with_name("files")
             .multiple(true)
             .value_name("FILES")
             .takes_value(true)
             .required_unless("stdin")
             .help("Path to one or more files of sample data"))
        .arg(Arg::with_name("alpha")
             .short("a")
             .long("alpha")
             .value_name("ALPHA")
             .help("Significance level α")
             .takes_value(true)
             .default_value(".05"))
        .arg(Arg::with_name("lax")
             .long("lax")
             .help("Ignore non-numeric input lines"))
        .arg(Arg::with_name("plot")
             .short("p")
             .long("plot")
             .help("Print standard boxplots"))
        .arg(Arg::with_name("ascii")
             .long("ascii")
             .help("Use only ASCII characters in boxplots"))
        .arg(Arg::with_name("width")
             .short("w")
             .long("width")
             .value_name("WIDTH")
             .takes_value(true)
             .help("Width of boxplot"))
        .get_matches();

    let ascii = matches.is_present("ascii");
    let lax_parsing = matches.is_present("lax");
    let draw_plot = matches.is_present("plot");
    let use_stdin = matches.is_present("stdin");

    let width = matches
        .value_of("width")
        .and_then(|w| w.parse::<usize>().ok())
        .or(term_size::dimensions().map(|(w, _)| w))
        .unwrap_or(80);

    let summaries = if use_stdin {
        vec![summarize_stdin(lax_parsing)]
    } else {
        matches.values_of("files").unwrap()
            .map(|f| summarize_file(f, lax_parsing))
            .collect()
    };

    match summaries.len() {
        0 => unreachable!(),
        2 => {
            let alpha = parse_alpha(matches.value_of("alpha").unwrap());

            display_t_test(
                &summaries[0],
                &summaries[1],
                alpha,
                draw_plot,
                width,
                ascii,
            );
        }
        _ => {
            if draw_plot {
                let summary_refs: Vec<&Summary> = summaries
                    .iter()
                    .collect();

                let plot = plot::comparison_plot(&summary_refs, width, ascii, true);
                println!("{}\n", plot);
            }

            for i in 0..summaries.len() {
                if i > 0 {
                    println!();
                }
                print_summary(&summaries[i]);
            }
        },
    };
}
