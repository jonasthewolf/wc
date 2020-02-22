use clap::{crate_authors, crate_description, crate_version, App, Arg, ArgMatches};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Error, ErrorKind};

//
// wc prints one line of counts for each file, and if the file was given as an argument,
// it prints the file name following the counts.
// If more than one file is given, wc prints a final line containing the cumulative counts, with the file name total.
// The counts are printed in this order: newlines, words, characters, bytes, maximum line length.
// Each count is printed right-justified in a field with at least one space between fields so that the numbers
// and file names normally line up nicely in columns.
// The width of the count fields varies depending on the inputs, so you should not depend on a particular field width.
// However, as a GNU extension, if only one count is printed, it is guaranteed to be printed without leading spaces.
//

#[derive(Debug)]
struct Metrics {
    bytes: usize,
    chars: usize,
    lines: usize,
    words: usize,
    max_line_length: usize,
    filename: String,
}

// Issues:
// - non-empty last line
// - performance (two iterations over line buffer)
// - missing bytes from BOM
// - too many allocations for buffer (at every iteration)
fn count(filename: &str) -> Result<Metrics, Error> {
    let mut m = Metrics {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
        filename: filename.to_owned(),
    };

    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);

    loop {
        let mut buffer = vec![];
        let count = reader.read_until(0xa, &mut buffer)?;
        if count == 0 {
            break;
        }
        m.bytes += count;
        let s =
            std::str::from_utf8(&buffer).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        m.words += s.split_whitespace().count();
        m.chars += s.chars().count();
        m.max_line_length = std::cmp::max(m.max_line_length, count);
        m.lines += 1;
    }

    Ok(m)
}

fn print_metrics(m: &Metrics, opts: &ArgMatches) {
    let f_l = opts.is_present("lines");
    let f_w = opts.is_present("words");
    let f_c = opts.is_present("chars");
    let f_b = opts.is_present("bytes");
    let f_m = opts.is_present("max_line_length");
    let def = !(f_c || f_w || f_b || f_m || f_l);
    if def || f_l {
        print!("{:>8}", m.lines);
    }
    if def || f_w {
        print!("{:>8}", m.words);
    }
    if f_c {
        print!("{:>8}", m.chars);
    }
    if def || f_b {
        print!("{:>8}", m.bytes);
    }
    if f_m {
        print!("{:>8}", m.max_line_length);
    }
    println!(" {}", m.filename);
}

fn calculate_total(ms: &[Metrics]) -> Metrics {
    let mut m = Metrics {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
        filename: "total".to_owned(),
    };

    for m_x in ms {
        m.bytes += m_x.bytes;
        m.chars += m_x.chars;
        m.lines += m_x.lines;
        m.words += m_x.words;
        m.max_line_length = std::cmp::max(m.max_line_length, m_x.max_line_length);
    }

    m
}

fn print_count(matches: &ArgMatches) -> Result<(), Error> {
    if let Some(files) = matches.values_of("files") {
        let mut total = vec![];
        for file in files {
            let m = count(&file)?;
            total.push(m);
        }
        for m in &total {
            print_metrics(&m, &matches);
        }
        if total.len() > 1 {
            let mut t = calculate_total(&total);
            t.filename = "total".to_owned();
            print_metrics(&t, &matches);
        }
    } else {
        // Stdin
    }

    Ok(())
}

// TODO: read from stdin if no files are given
// TODO: files0_from
// TODO: return value
fn main() {
    let matches = App::new("wc")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("bytes")
                .short('c')
                .long("bytes")
                .overrides_with("chars")
                .help("Print only the byte counts.")
        )
        .arg(
            Arg::with_name("chars")
                .short('m')
                .long("chars")
                .overrides_with("bytes")
                .help("Print only the character counts.")
        )
        .arg(
            Arg::with_name("words")
                .short('w')
                .long("words")
                .help("Print only the word counts.")
        )
        .arg(
            Arg::with_name("lines")
                .short('l')
                .long("lines")
                .help("Print only the newline counts.")
        )
        .arg(
            Arg::with_name("max_line_length")
                .short('L')
                .long("max-line-length")
                .help("Print only the maximum display widths. Tabs are set at every 8th column. Display widths of wide characters are considered. Non-printable characters are given 0 width.")
        )
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .takes_value(true)
                .multiple(true)
                .help("Sets the input file(s) to use.")
        )
        .arg(
            Arg::with_name("files0_from")
                .long("files0_from")
                .value_name("file")
                .takes_value(true)
                .multiple(false)
                .help("Disallow processing files named on the command line, and instead process those named in file file; each name being terminated by a zero byte (ASCII NUL). This is useful when the list of file names is so long that it may exceed a command line length limitation. In such cases, running wc via xargs is undesirable because it splits the list into pieces and makes wc print a total for each sublist rather than for the entire list. One way to produce a list of ASCII NUL terminated file names is with GNU find, using its -print0 predicate. If file is ‘-’ then the ASCII NUL terminated file names are read from standard input.")
    )
    .get_matches();

    std::process::exit(match print_count(&matches) {
        Err(_) => 1,
        Ok(_) => 0,
    });
}
