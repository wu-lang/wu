use super::lexer::TokenPosition;
use colored::Colorize;

#[derive(Debug)]
pub enum ResponseType {
    Wrong,
    Weird,
    Group(Vec<ResponseNode>),
}

#[derive(Debug)]
pub struct ResponseNode {
    pub position: Option<TokenPosition>,
    pub kind:     ResponseType,
    pub message:  String,
}

impl ResponseNode {
    pub fn display(&self, lines: &[String], path: &str) {
        let (color, kind) = match self.kind {
            ResponseType::Wrong => ("red",    "wrong"),
            ResponseType::Weird => ("yellow", "weird"),
            _                   => ("red",    "wrong"),
        };

        let message = format!(
            "{}{}{}\n",

            kind.color(color).bold(),
            ": ".white().bold(),
            self.message.bold(),
        );

        if let Some(ref position) = self.position {
            let line_number = if position.line == 0 {
                position.line
            } else {
                position.line - 1
            };

            let prefix = format!("{:5} |  ", line_number + 1).blue().bold();
            let line   = format!("{:5} {}\n{}{}", " ", "|".blue().bold(), prefix, lines.get(if line_number == 1 && lines.len() == 1 { 0 } else { line_number }).unwrap_or(lines.last().unwrap()));

            let indicator = format!(
                                "{:6}{}{:offset$}{:^<count$}", " ", "|".bold().blue(), " ", " ".color(color).bold(),
                                offset = position.col,
                                count  = 2,
                            );

            let path_line = format!("{:5}{}{}", " ", "--> ".blue().bold(), path);

            println!("{}{}\n{}\n{}", message, path_line, line, indicator)
        } else {
            if let ResponseType::Group(ref responses) = self.kind {
                for response in responses {
                    response.display(lines, path)
                }

                println!()
            }

            println!("{}", message);
        }
    }
}

pub fn make_error(position: Option<TokenPosition>, message: String) -> ResponseNode {
    ResponseNode {
        position,
        kind: ResponseType::Wrong,
        message,
    }
}

pub fn weird(position: Option<TokenPosition>, message: String, lines: &Vec<String>, path: &str) {
    let warning = ResponseNode {
        position,
        kind: ResponseType::Weird,
        message,
    };

    warning.display(lines, path)
}

pub type Response<T> = Result<T, ResponseNode>;
