use ledger_parser::{Ledger, ParseError};
extern crate ledger_parser;


fn main() {
    // Create a new LedgerFile object.
    let ledger_file: Result<Ledger, ParseError> = ledger_parser::parse("personal.ledger");
}
