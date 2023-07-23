use std::io::Error;

fn work() {
    let results = [Ok(1), Err("nope"), Ok(3), Err("bad")];

    let result = results.iter().cloned()
        .collect::<Result<Vec<_>, &str>>();

    // gives us the first error
    assert_eq!(Err("nope"), result);

    let results = [Ok(1), Ok(3)];

    let result: Result<Vec<_>, &str> = results.iter().cloned().collect();

    // gives us the list of answers
    assert_eq!(Ok(vec![1, 3]), result);
}