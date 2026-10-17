//! Numeric owner fields in the independent debugfs reports.

pub(super) fn owner_id(value: &str) -> Result<u32, std::num::ParseIntError> {
    // The saved VM debugfs report prints a high-bit group as signed decimal.
    // Accept that display only within i32, retaining the exact 32 stored bits;
    // positive displays must fit u32. Never truncate an out-of-range value.
    if value.starts_with('-') {
        value
            .parse::<i32>()
            .map(|signed| u32::from_le_bytes(signed.to_le_bytes()))
    } else {
        value.parse::<u32>()
    }
}

#[test]
fn signed_and_unsigned_owner_displays_preserve_all_bits() {
    // Actual first-profile report from focused CI run 39002666396.
    let report = "User: 1985229328   Group: -2023406815   Size: 18";
    for (label, expected) in [("User:", 0x7654_3210), ("Group:", 0x8765_4321)] {
        let token = report
            .split(label)
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap();
        assert_eq!(owner_id(token).unwrap(), expected);
    }
    for (text, expected) in [
        ("0", 0),
        ("2147483647", i32::MAX as u32),
        ("2147483648", 0x8000_0000),
        ("-2147483648", 0x8000_0000),
        ("4294967295", u32::MAX),
        ("-1", u32::MAX),
    ] {
        assert_eq!(owner_id(text).unwrap(), expected, "{text}");
    }
}

#[test]
fn malformed_and_out_of_range_owner_displays_are_refused() {
    for text in [
        "",
        "-2147483649",
        "4294967296",
        "-4294967296",
        "0x87654321",
        "123oops",
        "--1",
    ] {
        assert!(owner_id(text).is_err(), "accepted {text:?}");
    }
}
