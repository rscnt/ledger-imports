use ledger_parser::{Ledger, ParseError};
extern crate ledger_parser;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::NaiveDate;
use rust_decimal::Decimal;

struct DnbTransaction {
    date: NaiveDate,
    description: String,
    withdrawals: Decimal,
    deposits: Decimal,
}

fn read_transactions_from_csv(filename: &str) -> Result<Vec<DnbTransaction>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut transactions = vec![];

    for (i, line) in reader.lines().enumerate() {
        if i == 0 { continue; } // Skip header row
        let line = line?;
        let fields: Vec<&str> = line.split(',').collect();

        let date = NaiveDate::parse_from_str(fields[0], "%m/%d/%Y")?;
        let description = fields[1].to_string();
        let withdrawals = fields[3].parse::<Decimal>()?;
        let deposits = fields[4].parse::<Decimal>()?;

        let transaction = DnbTransaction { date, description, withdrawals, deposits };
        transactions.push(transaction);
    }

    Ok(transactions)
}


fn main() {
    // Create a new LedgerFile object.
    let transactions = read_transactions_from_csv("./dnb_transactions.csv");
    let ledger_file: Result<Ledger, ParseError> = ledger_parser::parse("personal.ledger");
}
