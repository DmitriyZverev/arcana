#[macro_export]
macro_rules! assert_file {
    ($path:expr, $expected:expr) => {
        pretty_assertions::assert_eq!(
            std::fs::read_to_string($path)?,
            $expected,
            "File content mismatch"
        );
    };
}

#[macro_export]
macro_rules! assert_cmd {
    ($output:expr, $expected:expr) => {
        let output = $output;
        let expected = $expected;

        pretty_assertions::assert_eq!(output.status.code(), expected.code, "Status code mismatch");
        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "Stdout mismatch"
        );
        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&expected.stderr),
            "Stderr mismatch"
        );
    };
}
