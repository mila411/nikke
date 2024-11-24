// src/main.rs

use nikke::parser::Parser;

fn main() {
    let sql = "INSERT INTO table_name (column1, column2) SELECT column1, column2 FROM table1 JOIN table2 ON table1.id = table2.id WHERE condition GROUP BY column1 HAVING COUNT(column1) > 1 ORDER BY column2 DESC;";

    match Parser::new(sql) {
        Ok(mut parser) => match parser.parse() {
            Ok(query) => {
                println!("Parsed Query:\n{:#?}", query);
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Failed to create parser: {}", e);
        }
    }
}
