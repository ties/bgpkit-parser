#![allow(deprecated)]

use bgpkit_parser::{models::ElemType, BgpkitParser};
use clap::{Parser, ValueEnum};
use std::io::Read;
use std::time::{Duration, Instant};

const DEFAULT_INPUT: &str =
    "http://archive.routeviews.org/bgpdata/2021.10/UPDATES/updates.20211001.0000.bz2";

/// Compare parser iterators over the same MRT input.
#[derive(Debug, Parser)]
#[command(about)]
struct Args {
    /// MRT file path or URL.
    #[arg(short, long, default_value = DEFAULT_INPUT)]
    input: String,

    /// Iterator to run.
    #[arg(short = 't', long, value_enum, default_value_t = IteratorChoice::All)]
    iterator: IteratorChoice,

    /// Maximum number of items to parse. Use 0 to parse the full input.
    #[arg(short, long, default_value_t = 1000)]
    limit: usize,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum IteratorChoice {
    All,
    Record,
    Elem,
    SharedElem,
    Update,
    Route,
    RawRecord,
}

impl IteratorChoice {
    fn selected(self) -> Vec<IteratorChoice> {
        match self {
            IteratorChoice::All => vec![
                IteratorChoice::Record,
                IteratorChoice::Elem,
                IteratorChoice::SharedElem,
                IteratorChoice::Update,
                IteratorChoice::Route,
                IteratorChoice::RawRecord,
            ],
            other => vec![other],
        }
    }

    fn label(self) -> &'static str {
        match self {
            IteratorChoice::All => "all",
            IteratorChoice::Record => "record",
            IteratorChoice::Elem => "elem",
            IteratorChoice::SharedElem => "shared-elem",
            IteratorChoice::Update => "update",
            IteratorChoice::Route => "route",
            IteratorChoice::RawRecord => "raw-record",
        }
    }
}

#[derive(Debug)]
struct RunStats {
    count: usize,
    elapsed: Duration,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let limit = match args.limit {
        0 => None,
        n => Some(n),
    };

    log::info!("Input: {}", args.input);
    log::info!(
        "Limit: {}",
        limit
            .map(|limit| limit.to_string())
            .unwrap_or_else(|| "none".to_string())
    );

    for iterator in args.iterator.selected() {
        let stats = run_iterator(iterator, &args.input, limit);
        log_stats(iterator.label(), &stats);
    }
}

fn run_iterator(iterator: IteratorChoice, input: &str, limit: Option<usize>) -> RunStats {
    match iterator {
        IteratorChoice::All => unreachable!("all expands before dispatch"),
        IteratorChoice::Record => {
            run_counted("record", input, limit, |parser| parser.into_record_iter())
        }
        IteratorChoice::Elem => run_counted("elem", input, limit, |parser| parser.into_elem_iter()),
        IteratorChoice::SharedElem => run_counted("shared-elem", input, limit, |parser| {
            parser.into_shared_elem_iter()
        }),
        IteratorChoice::Update => {
            run_counted("update", input, limit, |parser| parser.into_update_iter())
        }
        IteratorChoice::Route => run_route(input, limit),
        IteratorChoice::RawRecord => run_counted("raw-record", input, limit, |parser| {
            parser.into_raw_record_iter()
        }),
    }
}

fn run_counted<I, F>(label: &str, input: &str, limit: Option<usize>, build_iter: F) -> RunStats
where
    I: Iterator,
    F: FnOnce(BgpkitParser<Box<dyn Read + Send>>) -> I,
{
    log::info!("Running {label} iterator");

    let parser = BgpkitParser::new(input).expect("failed to create parser");
    let start = Instant::now();
    let count = count_with_limit(build_iter(parser), limit);

    RunStats {
        count,
        elapsed: start.elapsed(),
    }
}

fn run_route(input: &str, limit: Option<usize>) -> RunStats {
    log::info!("Running route iterator");

    let parser = BgpkitParser::new(input).expect("failed to create parser");
    let start = Instant::now();

    let mut count = 0usize;
    let mut announces = 0usize;
    let mut withdrawals = 0usize;

    for route in parser.into_route_iter() {
        if count < 3 {
            log::info!(
                "Route {}: {:?} {} via AS{} (peer: {}, path: {:?})",
                count + 1,
                route.elem_type,
                route.prefix,
                route.peer_asn,
                route.peer_ip,
                route.as_path.as_ref().map(|path| path.to_string())
            );
        }

        match route.elem_type {
            ElemType::ANNOUNCE => announces += 1,
            ElemType::WITHDRAW => withdrawals += 1,
        }

        count += 1;
        if matches!(limit, Some(limit) if count >= limit) {
            break;
        }
    }

    log::info!("Route announcements: {announces} | withdrawals: {withdrawals}");

    RunStats {
        count,
        elapsed: start.elapsed(),
    }
}

fn count_with_limit<I>(iter: I, limit: Option<usize>) -> usize
where
    I: Iterator,
{
    match limit {
        Some(limit) => iter.take(limit).count(),
        None => iter.count(),
    }
}

fn log_stats(label: &str, stats: &RunStats) {
    let elapsed_secs = stats.elapsed.as_secs_f64();
    log::info!(
        "{label}: {} items in {:.3}s ({:.0} items/s)",
        stats.count,
        elapsed_secs,
        stats.count as f64 / elapsed_secs
    );
}
