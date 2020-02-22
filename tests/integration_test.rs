
use std::process::Command;

const GNU_WC: &str = "wc";
const MY_WC: &str = "./target/debug/wc";

#[cfg(test)]
fn compare_file_to_gnu_wc(file: &[&str], args: &[&str]) {
    let wc_struct = dbg!(Command::new(GNU_WC)
                                        .args(args)
                                        .args(file))
                                        .output()
                                        .expect("wc not found.");
    let wc_out = std::str::from_utf8(wc_struct.stdout.as_ref()).unwrap();
    let wc_status = wc_struct.status;
    let my_struct = dbg!(Command::new(MY_WC)
                                        .args(args)
                                        .args(file))
                                        .output()
                                        .expect("my wc not found");
    let my_out = std::str::from_utf8(my_struct.stdout.as_ref()).unwrap();
    let my_status = my_struct.status;
    assert_eq!(my_status, wc_status);
    assert_eq!(my_out,  wc_out);
}

#[test]
fn compare_simple() {
    compare_file_to_gnu_wc(&["src/main.rs"], &[]);
}

#[test]
fn compare_simple_flags() {
    compare_file_to_gnu_wc(&["src/main.rs"], &["-c"]);
}
#[test]
fn compare_simple_flags2() {
    compare_file_to_gnu_wc(&["src/main.rs"], &["-cml"]);
}
#[test]
fn compare_simple_flags3() {
    compare_file_to_gnu_wc(&["src/main.rs"], &["-mc", "-l"]);
}
#[test]
fn file_not_found() {
    compare_file_to_gnu_wc(&["file_should_not_exist"], &[]);
}