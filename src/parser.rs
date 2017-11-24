#[allow(unused_imports)]
use accounting::{Account, Transaction, Posting, Amount, CommoditySymbol};
use lexer::{Token};
use lexer::TokenType::*;
use error::error;
use chrono;
use chrono::Utc;
use status::Status as TxStatus;
use unicode_segmentation::UnicodeSegmentation;

// Now to confabulate these disgraced and shattered things
pub fn parse<'a>(tokens: Vec<Token>) -> Vec<Posting> {
    let mut postings: Vec<Posting> = Vec::new();

    let mut parser = Parser::new(tokens);
    let token = parser.advance();
    println!("{:?}", token);

    postings
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens: tokens, current: 0 }
    }
    fn is_at_end(&self) -> bool {
        self.tokens[self.current + 1].token_type() == &EOF
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous().clone()
    }
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}



// fn match_token() -> {
//     match token.token_type() {
//         &Date => {
//             // Reset
//             if date.is_some() {
//                 date = None;
//                 status = TxStatus::Unmarked;
//                 description = None;
//                 transaction = None;
//                 current_account = None;
//             }
//
//             let date_string = token.literal();
//             let mut naive_date = chrono::NaiveDate::parse_from_str(date_string, "%Y-%m-%d");
//             if naive_date.is_ok() {
//                 date = Some(chrono::Date::from_utc(naive_date.unwrap(), chrono::Utc));
//             }
//             else {
//                 naive_date = chrono::NaiveDate::parse_from_str(date_string, "%Y-%m-%d");
//             }
//             if naive_date.is_ok() {
//                 date = Some(chrono::Date::from_utc(naive_date.unwrap(), chrono::Utc));
//             }
//             else {
//                 error(token.line(), "Date is not parseable");
//             }
//         }
//         &Status => {
//             if date.is_none() { error(token.line(), "No Date for Status"); }
//
//             // TODO: Would be nice to revise to not use the catch-all matching
//             let status_string = &token.literal();
//             status = match &status_string[..] {
//                 "!" => { TxStatus::Pending }
//                 "*" => { TxStatus::Cleared }
//                 _ => { TxStatus::Unmarked }
//             }
//         }
//         &Description => {
//             if date.is_none() { error(token.line(), "No Date for Description"); }
//
//             description = Some(token.literal().clone());
//
//             transaction = Some(Transaction::new(token.line(), date.unwrap(), status, description.unwrap()));
//         }
//         &Indentation => {
//             if date.is_none() { error(token.line(), "Stray indentation"); }
//         }
//         &AccountName => {
//             let literal = token.literal();
//             current_account = Some(Account::new(literal.clone()));
//         }
//         &Currency => {
//             let literal = token.literal().clone();
//             let split_words = literal.split_word_bounds().collect::<Vec<&str>>();
//             let iterable_words = split_words.iter();
//             let mut currency: Option<CommoditySymbol> = None;
//             let mut is_negative = false;
//             let mut starts_with_dot = false;
//             let mut amount: f64 = 0f64;
//
//             for word in iterable_words {
//                 match word {
//                     &"$" => {
//                         currency = Some(word.to_string())
//                     }
//                     &"-" => {
//                         is_negative = true;
//                     }
//                     &"." => {
//                         starts_with_dot = true;
//                     }
//                     &number => {
//                         let float_str: String =
//                             if starts_with_dot == true {
//                                 let mut s = ".".to_string();
//                                 s.push_str(number);
//                                 s
//                             }
//                             else {
//                                 number.to_string()
//                             };
//                         let parsed = float_str.parse();
//                         match parsed {
//                             Ok(parsed) => {
//                                 amount = parsed;
//                             }
//                             Err(err) => {
//                                 error(token.line(), &format!("{}. Could not parse {}", err, literal))
//                             }
//                         }
//                     }
//                 }
//             }
//             if is_negative == true {
//                 amount = -amount;
//             }
//
//             let amount = Amount::new(currency.unwrap(), amount);
//             postings.push(Posting::new(transaction.clone().unwrap(), current_account.clone().unwrap(), Some(amount)));
//         }
//         &CurrencyInferred => {
//             postings.push(Posting::new(transaction.clone().unwrap(), current_account.clone().unwrap(), None));
//         }
//         &EOF => {
//             // Done
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_float() {
        assert_eq!("04".parse::<f64>(), Ok(4f64));
        assert_eq!(".04".parse::<f64>(), Ok(0.04));
    }


    // #[should_panic]
    // fn test_panic_on_newline_with_indentation() {
        // parse(vec![TokenType::Indentation]);
    // }
}
