use config::{Config, Value};

/// Gets an array from a config file using a key, returning `None` if
/// the key wasn't present or couldn't be coerced into a `Vec<Value>`.
///
/// `Config::get_array()` in version `0.4.1` has a bug where it
/// consumes `self` instead of taking `self` by reference. This
/// function is a workaround for the time being. The bug has been
/// fixed but will not be available until the release of version `0.5`
/// of `config-rs`.
pub fn ref_get_array(key: &str, config: &Config) -> Option<Vec<Value>> {
    config.get(key).and_then(Value::into_array)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // The real test is that this compiles successfully without giving
    // a warning about the config file being moved.
    fn test_ref_get_array() {
        let mut c: Config = Config::new();
        c.set("numbers", vec![1, 2, 3]).unwrap();
        c.set("boolean", true).unwrap();

        // Uncomment and the compiler should complain
        // assert_eq!(c.get_array("numbers"),
        //            Some(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
        // assert_eq!(c.get_array("boolean"), None); // Use of moved value: c

        assert_eq!(ref_get_array("numbers", &c),
                   Some(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]));
        assert_eq!(ref_get_array("boolean", &c), None);
    }
}
