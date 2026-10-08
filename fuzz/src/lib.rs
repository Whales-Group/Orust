//! Deterministic parser corpus used by the CI fuzz-smoke gate.

#[cfg(test)]
mod tests {
    use orust_syntax::DocBlock;

    #[test]
    fn arbitrary_comment_corpus_never_panics_or_loses_the_raw_text() {
        let corpus = [
            "",
            "@",
            "/// plain text\n/// @unknown value",
            "/** nested /* block */ text */",
            "/// ```orust\n/// @inside\n/// ```",
            "😀\r\n\t@tag with punctuation !?",
        ];
        for input in corpus {
            let block = DocBlock::parse(
                input,
                orust_syntax::Span {
                    start: 0,
                    end: input.len(),
                },
            );
            assert_eq!(block.raw, input);
        }
    }
}
