#![deny(warnings)]

#[macro_use]
extern crate clap;
extern crate csv;
#[macro_use]
extern crate derive_error;
extern crate framed;
extern crate serde_json;

include!(env!("FRAMED_DECODE_DYNAMIC_RS"));

mod error;
use error::{Error, Result};

use clap::Arg;
use framed::typed::Receiver;
use std::io::{stdin, stdout};

arg_enum! {
    #[derive(Debug, Eq, PartialEq)]
    enum OutputFormat {
        Csv,
        Debug,
        Json
    }
}

fn main() {
    match try() {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}\n\
                             Detail: {:#?}", e, e),
    };
}

fn try() -> Result<()> {
    let app = app_from_crate!()
              .arg(Arg::with_name("out-format")
                       .long("out-format")
                       .help("Output format type used to write data to stdout.")
                       .takes_value(true)
                       .empty_values(false)
                       .possible_values(&OutputFormat::variants())
                       .default_value("Debug")
                       .case_insensitive(true));
    let matches = app.get_matches();
    let out_fmt = value_t!(matches, "out-format", OutputFormat)?;

    let mut r = Receiver::<_, UserType>::new(stdin());

    let mut csvw: Option<csv::Writer<_>> =
        match out_fmt {
            OutputFormat::Csv =>
                Some(csv::WriterBuilder::new().from_writer(stdout())),
            _ => None,
        };

    loop {
        let res = r.recv();
        match res {
            Ok(v) => match out_fmt {
                OutputFormat::Csv => csvw.as_mut()
                                         .expect("Should've been initialized")
                                         .serialize(&v)?,
                OutputFormat::Debug => println!("{:#?}", v),
                OutputFormat::Json => {
                    serde_json::to_writer(stdout(), &v)?;
                    println!("");
                },
            },
            Err(framed::Error::EofBeforeFrame) => return Ok(()),
            Err(e) => return Err(Error::from(e)),
        };
    }

    // Not reached.
}
