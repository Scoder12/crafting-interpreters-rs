use std::io::Write;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: {} <script>", args.get(0).unwrap_or(&String::new()));
        std::process::exit(1);
    }
    if args.len() == 2 {
        run_file(args.get(1).unwrap());
        std::process::exit(0);
    }
    run_prompt();
}

fn run_file(filename: &str) {
    let bytes = std::fs::read(filename).expect("file should read");
    run(String::from_utf8(bytes).expect("file should contain valid utf-8"));
}

fn run_prompt() {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        run(line);
    }
}

fn run(line: String) {
    let tokens = parser::lexer::lex(&line);
    println!("{:#?}", tokens);
    let parse = parser::parser::parse(tokens);
    println!("{:#?}", parse.syntax());
    println!("errors: {:?}", parse.errors);
}
