
use std::ops::Range;

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use chumsky::error::{Rich, RichPattern};

use crate::lexer::{LexError, SpannedToken, Token};

const FILE_NAME: &str = "input.vis";


fn render(
    source: &str,
    span: Range<usize>,
    message: &str,
    label: &str,
    note: Option<&str>,
) {
    let span = span.start.min(source.len())..span.end.min(source.len());

    let mut report = Report::build(ReportKind::Error, (FILE_NAME, span.clone()))
        .with_config(Config::new().with_index_type(IndexType::Byte))
        .with_message(message)
        .with_label(
            Label::new((FILE_NAME, span))
                .with_message(label)
                .with_color(Color::Red),
        );

    if let Some(note) = note {
        report = report.with_note(note);
    }

    report
        .finish()
        .eprint((FILE_NAME, Source::from(source)))
        .expect("failed to render diagnostic");
}


pub fn render_lex_error(source: &str, error: &LexError) {
    render(
        source,
        error.span.clone(),
        "Invalid character",
        &format!("unexpected character {:?}", error.lexeme),
        None,
    );
}


pub fn render_parse_error(
    source: &str,
    tokens: &[SpannedToken],
    error: &Rich<'_, Token>,
) {
    let token_range = error.span().into_range();

    let source_span =
        token_range_to_source(&token_range, tokens, source.len());

    let found = match error.found() {
        Some(token) => format!("{token:?}"),
        None => "end of input".to_owned(),
    };

    let labels: Vec<String> = error
        .expected()
        .filter_map(|pattern| match pattern {
            RichPattern::Label(label) => Some(label.to_string()),
            _ => None,
        })
        .collect();

    let expected: Vec<String> = if labels.is_empty() {
        error
            .expected()
            .map(|pattern| format!("{pattern:?}"))
            .collect()
    } else {
        labels
    };

    let note = if expected.is_empty() {
        None
    } else {
        Some(format!("expected {}", expected.join(", ")))
    };

    render(
        source,
        source_span,
        "Syntax error",
        &format!("found {found}"),
        note.as_deref(),
    );
}


fn token_range_to_source(
    token_range: &Range<usize>,
    tokens: &[SpannedToken],
    source_len: usize,
) -> Range<usize> {
    let start = token_range.start.min(tokens.len());
    let end = token_range.end.min(tokens.len());

    if start < end {
        tokens[start].span.start..tokens[end - 1].span.end
    } else {
        // An empty span generally means the parser expected another
        // token here, often at the end of the input.
        let offset = tokens
            .get(start)
            .map(|token| token.span.start)
            .unwrap_or(source_len);

        offset..offset
    }
}
