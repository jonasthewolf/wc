use clap::{crate_authors, crate_description, crate_version, App, Arg, ArgMatches};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, Error, ErrorKind};

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

#[derive(Clone, Debug)]
struct Metrics {
    bytes: usize,
    chars: usize,
    lines: usize,
    words: usize,
    max_line_length: usize,
    filename: String,
}

struct ShowOptions {
    lines: bool,
    chars: bool,
    bytes: bool,
    words: bool,
    max_line_length: bool,
}

impl ShowOptions {
    fn from_clap_matches(opts: &ArgMatches) -> ShowOptions {
        ShowOptions {
            lines: opts.is_present("lines"),
            words: opts.is_present("words"),
            chars: opts.is_present("chars"),
            bytes: opts.is_present("bytes"),
            max_line_length: opts.is_present("max_line_length"),
        }
    }

    fn is_default(&self) -> bool {
        !(self.chars || self.words || self.bytes || self.max_line_length || self.lines)
    }
}


// TODO: missing bytes from BOM?
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
    let mut reader = BufReader::with_capacity(1024, f);
    loop {
        let buffer = reader.fill_buf()?;
        let mut last_char_was_word_separator  = None;
        let mut bytes = 0;
        if buffer.is_empty() {
            break;
        }
        let s =
            std::str::from_utf8(&buffer).map_err(|err| Error::new(ErrorKind::InvalidData, err))?;
        for c in s.chars() {
            bytes += c.len_utf8();
            m.chars += 1;
            if c == '\n' {
                m.lines += 1;
                break;
            } else if c.is_whitespace() {
                last_char_was_word_separator = Some(true);
                if c ==  '\t' {
                    m.max_line_length += 7;
                }
            } else {
                if let Some(true) = last_char_was_word_separator {
                    m.words += 1;
                }
                last_char_was_word_separator = Some(false);
            }
        }
        if let Some(false) = last_char_was_word_separator {
            m.words += 1;
        }
        m.bytes += bytes;
        reader.consume(bytes);

        // TODO m.max_line_length = std::cmp::max(m.max_line_length, count);

    }

    Ok(m)
}

fn print_metrics(out: &mut dyn io::Write,  m: &Metrics, opts: &ShowOptions, mwpc: &Metrics) {
    let mut remove_column =
    if opts.is_default() || opts.lines {
        write!(out, "{:>width$} ", m.lines, width = mwpc.lines).unwrap();
        1
    } else {
        0
    };
    if opts.is_default() || opts.words {
        write!(out, "{:>width$} ", m.words, width = mwpc.words-remove_column).unwrap();
        remove_column = 1;
    }
    if opts.chars {
        write!(out, "{:>width$} ", m.chars, width = mwpc.chars-remove_column).unwrap();
        remove_column = 1;
    }
    if opts.is_default() || opts.bytes {
        write!(out, "{:>width$} ", m.bytes, width = mwpc.bytes-remove_column).unwrap();
        remove_column = 1;
    }
    if opts.max_line_length {
        write!(out, 
            "{:>width$} ",
            m.max_line_length,
            width = mwpc.max_line_length - remove_column
        ).unwrap();
    }
    writeln!(out, "{}", m.filename).unwrap();
}

fn calculate_total_and_max_width_per_column(ms: &[Metrics]) -> (Metrics, Metrics) {
    let mut total = Metrics {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
        filename: "total".to_owned(),
    };
    let mut mwpc = Metrics {
        bytes: 0,
        chars: 0,
        lines: 0,
        words: 0,
        max_line_length: 0,
        filename: "".to_owned(), // Width of filename is not important
    };
    for m_x in ms {
        total.bytes += m_x.bytes;
        total.chars += m_x.chars;
        total.lines += m_x.lines;
        total.words += m_x.words;
        total.max_line_length = std::cmp::max(total.max_line_length, m_x.max_line_length);
        mwpc.bytes = std::cmp::max(mwpc.bytes, m_x.bytes);
        mwpc.chars = std::cmp::max(mwpc.chars, m_x.chars);
        mwpc.lines = std::cmp::max(mwpc.lines, m_x.lines);
        mwpc.words = std::cmp::max(mwpc.words, m_x.words);
        // mwpc.max_line_length not needed again
    }
    mwpc.bytes = std::cmp::max(mwpc.bytes.to_string().len(), 8);
    mwpc.chars = std::cmp::max(mwpc.chars.to_string().len(), 8);
    mwpc.lines = std::cmp::max(mwpc.lines.to_string().len(), 8);
    mwpc.words = std::cmp::max(mwpc.words.to_string().len(), 8);
    // TODO what to do with mwpc.max_line_length?
    (total, mwpc)
}

fn print_count(mut out: &mut dyn io::Write, matches: &ArgMatches) -> Result<(), Error> {
    let opts = ShowOptions::from_clap_matches(matches);
    if let Some(files) = matches.values_of("files") {
        let mut all_metrics = vec![];
        for file in files {
            let m = count(&file)?;
            all_metrics.push(m);
        }
        let (total, mwpc) = calculate_total_and_max_width_per_column(&all_metrics);
        for m in &all_metrics {
            print_metrics(&mut out, &m, &opts, &mwpc);
        }
        if all_metrics.len() > 1 {
            print_metrics(&mut out, &total, &opts, &mwpc);
        }
    } else {
        // Stdin
    }

    Ok(())
}

// TODO: read from stdin if no files are given
// TODO: files0_from
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

    std::process::exit(match print_count(&mut io::stdout().lock(), &matches) {
        Err(_) => 1,
        Ok(_) => 0,
    });
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn sample_metrics() -> Vec<Metrics> {
        let m0 = Metrics {
            bytes: 123,
            chars: 9_876_543_210,
            lines: 789,
            words: 1_239_875_670,
            max_line_length: 456,
            filename: "m0".to_owned(),
        };
        let m1 = Metrics {
            bytes: 1_234_567_890,
            chars: 1_234_567,
            lines: 12_345_678_901,
            words: 4_567_890,
            max_line_length: 4_567_890_123,
            filename: "m1".to_owned(),
        };
        vec![m0, m1]
    }

    #[test]
    fn wider_columns() {
        let metrics = sample_metrics();
        let m0 = metrics[0].clone();
        let m1 = metrics[1].clone();
        let (total, mwpc) = calculate_total_and_max_width_per_column(&[m0.clone(), m1.clone()]);
        assert_eq!(total.bytes, m0.bytes + m1.bytes);
        assert_eq!(total.chars, m0.chars + m1.chars);
        assert_eq!(total.lines, m0.lines + m1.lines);
        assert_eq!(total.words, m0.words + m1.words);
        assert_eq!(
            total.max_line_length,
            std::cmp::max(m0.max_line_length, m1.max_line_length)
        );
        assert_eq!(mwpc.bytes, 10);
        assert_eq!(mwpc.chars, 10);
        assert_eq!(mwpc.lines, 11);
        assert_eq!(mwpc.words, 10);
        //assert_eq!(mwpc.max_line_length, 10);
    }

    #[test]
    fn print_wide_columns() {
        let metrics = sample_metrics();
        let m0 = metrics[0].clone();
        let m1 = metrics[1].clone();
        let mwpc = Metrics {
            bytes: 10,
            chars: 10,
            lines: 11,
            words: 10,
            max_line_length: 10,
            filename: "m1".to_owned(),
        };
        let opts = ShowOptions {
            lines: true,
            chars: true,
            bytes: false,
            words: true,
            max_line_length: true,
        };
        let mut writer = vec![];
        print_metrics(&mut writer, &m0, &opts, &mwpc);
        print_metrics(&mut writer, &m1, &opts, &mwpc);
        let output = std::str::from_utf8(writer.as_ref()).unwrap();
        assert_eq!(dbg!(output), "        789 1239875670 9876543210       456 m0\n12345678901   4567890   1234567 4567890123 m1\n")
    }
}
