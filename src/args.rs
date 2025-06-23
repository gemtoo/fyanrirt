use crate::engine::SMPPCredentials;
use crate::engine::SMS;
use clap::Command;
use clap::{command, Arg, ArgMatches};
use rusmpp::types::COctetString;
use std::{error::Error, str::FromStr};

fn from_cli() -> ArgMatches {
    let args = command!()
        .arg(
            Arg::new("provider-name")
                .long("smsc-name")
                .help("SMS center name. For example A1, MTS.")
                .required(true),
        )
        // .arg(
        //     Arg::new("rate-limit")
        //         .long("rate-per-minute")
        //         .help("Maximum amount of SMS per minute allowed by the provider.")
        //         .required(true),
        // )
        .arg(
            Arg::new("smpp-socket-address")
                .long("endpoint")
                .help("SMPP server address to connect to. For example 203.0.113.32:2775, sms.example.com:1234.")
                .required(true),
        )
        .arg(
            Arg::new("sys-id")
                .long("system-id")
                .help("System ID of an SMS center. Maximum string length is 16 characters.")
                .required(true),
        )
        .arg(
            Arg::new("password")
                .long("password")
                .help("Password for authentication. Maximum string length is 9 characters.")
                .required(true),
        )
        .arg(
            Arg::new("sys-type")
                .long("system-type")
                .help("SMPP server system type. Maximum string length is 13 characters.")
                .required(true),
        )
        .subcommand(
            Command::new("send-sms")
                .about("Send an SMS")
                .arg(
                    Arg::new("src")
                        .long("src")
                        .help("Source name.")
                        .required(true),
                )
                .arg(
                    Arg::new("dst")
                        .long("dst")
                        .help("Destination phone number.")
                        .required(true),
                )
                .arg(
                    Arg::new("content")
                        .long("content")
                        .help("Text content of the SMS.")
                        .required(true),
                ),
        ).subcommand_required(true)
        .get_matches();

    return args;
}

pub fn credentials() -> Result<SMPPCredentials, Box<dyn Error>> {
    let args = from_cli();
    let provider_name = args.get_one::<String>("provider-name").unwrap().clone();
    // let rate_limit_string = args.get_one::<String>("rate-limit").unwrap();
    // let rate_limit = rate_limit_string.parse::<u64>()?;
    let endpoint_addr = args
        .get_one::<String>("smpp-socket-address")
        .unwrap()
        .clone();
    let system_id_raw = args.get_one::<String>("sys-id").unwrap();
    let password_raw = args.get_one::<String>("password").unwrap();
    let systype_raw = args.get_one::<String>("sys-type").unwrap();

    let system_id: COctetString<1, 16> = COctetString::from_str(system_id_raw)?;
    let password: COctetString<1, 9> = COctetString::from_str(password_raw)?;
    let system_type: COctetString<1, 13> = COctetString::from_str(systype_raw)?;

    let credentials = SMPPCredentials {
        provider_name,
        endpoint_addr,
        system_id,
        password,
        system_type,
        //    rate_limit,
    };
    Ok(credentials)
}

pub fn smsoneshot() -> SMS {
    let args = from_cli();
    if let Some(sub_matches) = args.subcommand_matches("send-sms") {
        let src = sub_matches.get_one::<String>("src").unwrap().clone();
        let dst = sub_matches.get_one::<String>("dst").unwrap().clone();
        let content = sub_matches.get_one::<String>("content").unwrap().clone();
        SMS { src, dst, content }
    } else {
        // unreachable because clap requires this
        unreachable!();
    }
}
