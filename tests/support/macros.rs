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
macro_rules! format_cmd_output {
    ($code:expr, $stdout:expr, $stderr:expr) => {
        format!(
            "--------------\nexit code: {}\n--- stdout ---\n{}\n--- stderr ---\n{}\n--------------",
            $code.map_or("none".to_string(), |c| c.to_string()),
            String::from_utf8_lossy($stdout),
            String::from_utf8_lossy($stderr),
        )
    };
}

#[macro_export]
macro_rules! assert_cmd {
    ($output:expr, $expected:expr) => {
        let output = $output;
        let expected = $expected;

        pretty_assertions::assert_eq!(
            format_cmd_output!(&output.status.code(), &output.stdout, &output.stderr),
            format_cmd_output!(&expected.code, &expected.stdout, &expected.stderr),
            "Command output mismatch"
        );
    };
}
