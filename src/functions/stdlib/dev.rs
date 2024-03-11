use crate::{define_stdfunction, Error};
use polyvalue::Value;
use std::io::BufRead;

define_stdfunction!(
    time { },
    returns = Float,
    docs = {
        category: "Development",
        description: "Returns a unix timestamp for the current system time",
        ext_description: "
            Returns a unix timestamp for the current system time.
            The timestamp is a floating point number representing the number of seconds since the Unix epoch.",
        examples: "
            assert(
                time() > 0
            )
        "
    },
    handler = (_state, _reference) {
        Ok(Value::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::ZERO)
                .as_secs_f64(),
        ))
    }
);

define_stdfunction!(
    tail {
        file: Standard::String,
        lines: Optional::Int
    },
    returns = Array,
    docs = {
        category: "Development",
        description: "Returns the last <lines> lines from a given file",
        ext_description: "
            If <lines> is not specified, the function will return the last line of the file.",
        examples: "
            lines = tail('.gitignore')
            assert_eq(
                lines,
                ['/Cargo.lock']
            )
        "
    },
    handler = (state, _reference) {
        let n = optional_arg!(state::lines).unwrap_or(1.into()).as_a::<i64>()?;
        let file = required_arg!(state::file).to_string();

        let file = std::fs::File::open(file)?;
        let lines = std::io::BufReader::new(file)
            .lines()
            .map(|f| Ok::<Value, Error>(Value::from(f?)))
            .collect::<Result<Vec<_>, _>>()?;

        // return last n
        Ok(Value::from(lines.iter().rev().take(n as usize).rev().cloned().collect::<Vec<_>>()))
    }
);
