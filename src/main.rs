use std::env;
use ledger_parser::{Ledger};
use regex::Regex;

use std::fmt;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, read_to_string};
use std::collections::HashSet;
use chrono::NaiveDate;
use rust_decimal::Decimal;

struct DnbTransaction {
    date: NaiveDate,
    description: String,
    account_from: String,
    account_to: String,
    withdrawals: Option<Decimal>,
    deposits: Option<Decimal>,
}

impl fmt::Display for DnbTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut transaction_str = format!(
            "{} {}",
            self.date.format("%Y-%m-%d"),
            self.description
        );
        if let Some(withdrawals) = self.withdrawals {
            transaction_str.push_str(&format!("\n\tExpenses:{}\tNOK{}", self.account_to, withdrawals));
            transaction_str.push_str(&format!("\n\t{}", self.account_from));
        }
        if let Some(deposits) = self.deposits {
            transaction_str.push_str(&format!("\n\t{}\tNOK{}", self.account_from, deposits));
            transaction_str.push_str(&format!("\n\tIncome:{}", self.account_to));
        }
        write!(f, "{}", transaction_str)
    }
}

fn parse_decimal(value: &str) -> Option<Decimal> {
    match value.parse::<Decimal>() {
        Ok(decimal) => Some(decimal),
        Err(_) => None,
    }
}

fn clean_description_text(description: &str) -> String {
    let mut_description = String::from(description);
    let skien_dato_regex_pattern = Regex::new(r"Dato.*").unwrap();
    skien_dato_regex_pattern.replace(&mut_description, "").to_string()
}

fn read_transactions_from_csv(filename: &str) -> Result<Vec<DnbTransaction>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut transactions = vec![];
    let mut csvrdr = csv::Reader::from_reader(reader);

    for result in csvrdr.records() {
        let record = result?;
        let date = NaiveDate::parse_from_str(&record[0], "%m/%d/%Y")?;
        let description = record[1].to_string();
        let account_to = clean_description_text(&description.replace(" ", "_").to_string());
        let account_from = String::from("Assets:DNB:Checking");
        let withdrawals = parse_decimal(&record[3]);
        let deposits = parse_decimal(&record[4].to_string().replace(",", "").replace("\"", ""));

        let transaction = DnbTransaction { date, description, withdrawals, deposits, account_from, account_to };
        transactions.push(transaction);
    }

    Ok(transactions)
}

fn parse_ledger_file(filename: &str) -> Result<Ledger, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    match ledger_parser::parse(&read_to_string(reader)?) {
        Ok(ledger_file) => Ok(ledger_file),
        Err(err) => Err(Box::new(err))
    }
}

fn import_dnb_transactions(ledger_file: Ledger, dnb_txns: Vec<DnbTransaction>) {
    for dnb_tnx in dnb_txns {
        let transaction_str = dnb_tnx.to_string();
        let ledger_entry = format!("{}\n", transaction_str);
        println!("{}", ledger_entry);
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let m = s1.chars().count();
    let n = s2.chars().count();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }

    for j in 0..=n {
        dp[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            if c1 == c2 {
                dp[i + 1][j + 1] = dp[i][j];
            } else {
                dp[i + 1][j + 1] = 1 + dp[i][j].min(dp[i + 1][j]).min(dp[i][j + 1]);
            }
        }
    }

    dp[m][n]
}

fn group_strings_by_levenshtein(strings: &[String], threshold: usize) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();

    for string in strings {
        let mut found_group = false;

        for group in &mut groups {
            for existing_string in group.iter() {
                if levenshtein_distance(string, existing_string) <= threshold {
                    group.push(string.clone());
                    found_group = true;
                    break;
                }
            }

            if found_group {
                break;
            }
        }

        if !found_group {
            groups.push(vec![string.clone()]);
        }
    }

    groups
}


fn main() {
    // Create a new LedgerFile object.
    let args: Vec<String> = env::args().skip(1).collect();
    match read_transactions_from_csv(&args[0]) {
        Ok(transactions) => {
            let descriptions: Vec<String> = transactions.iter().map ( |tx| String::from(&tx.account_to) ).collect();
            let description_groups = group_strings_by_levenshtein(&descriptions, 5);
            match parse_ledger_file(&args[1]) {
                Ok(ledger_file) => {
                    import_dnb_transactions(ledger_file, transactions);
                }
                Err(err) => {
                    println!("Error :: {:?}", err)
                }
            }
        }
        Err(err) => {
            println!("Error reading dnb transactions: {}", err)
        }
    }
}
