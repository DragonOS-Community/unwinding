#![no_std]
#![feature(start)]

extern crate alloc;
extern crate unwinding;

use alloc::{borrow::ToOwned, string::String};
use unwinding::print::*;

#[link(name = "c")]
unsafe extern "C" {}

struct PrintOnDrop(String);

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        eprintln!("dropped: {:?}", self.0);
    }
}

struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("panic on drop");
    }
}

#[track_caller]
fn foo() {
    panic!("panic");
}

fn bar() {
    let _p = PrintOnDrop("string".to_owned());
    foo()
}

fn main() {
    let _ = unwinding::panic::catch_unwind(|| {
        bar();
        eprintln!("done");
    });
    eprintln!("caught");
    let _p = PanicOnDrop;
    foo();
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    unwinding::panic::catch_unwind(|| {
        main();
        0
    })
    .unwrap_or(101)
}
