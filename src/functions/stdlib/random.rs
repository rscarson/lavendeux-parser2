use crate::define_stdfunction;
use polyvalue::{types::Range, Value, ValueTrait};

define_stdfunction!(
    sha256 {
        input: Standard::String
    },
    returns = String,
    docs = {
        category: "Cryptographic",
        description: "Returns the sha256 hash of a given string",
        ext_description: "Will return an unsalted sha256 hash of the input string.",
        examples: "
            assert_eq(
                sha256('hello'),
                '2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824'
            )
        "
    },
    handler = (state) {
        let input = required_arg!(state::input).to_string();

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(input);

        let s = format!("{:X}", hasher.finalize());
        Ok(Value::from(s))
    }
);

define_stdfunction!(
    sha512 {
        input: Standard::String
    },
    returns = String,
    docs = {
        category: "Cryptographic",
        description: "Returns the sha512 hash of a given string",
        ext_description: "Will return an unsalted sha512 hash of the input string.",
        examples: "
            assert_eq(
                sha512('hello'),
                '9B71D224BD62F3785D96D46AD3EA3D73319BFBC2890CAADAE2DFF72519673CA72323C3D99BA5C11D7C7ACC6E14B8C5DA0C4663475C2E5C3ADEF46F73BCDEC043'
            )
        "
    },
    handler = (state) {
        let input = required_arg!(state::input).to_string();

        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(input);

        let s = format!("{:X}", hasher.finalize());
        Ok(Value::from(s))
    }
);

define_stdfunction!(
    md5 {
        input: Standard::String
    },
    returns = String,
    docs = {
        category: "Cryptographic",
        description: "Returns the md5 hash of a given string",
        ext_description: "Will return an unsalted md5 hash of the input string.",
        examples: "
            assert_eq(
                md5('hello'),
                '5D41402ABC4B2A76B9719D911017C592'
            )
        "
    },
    handler = (state) {
        let input = required_arg!(state::input).to_string();

        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(input);

        let s = format!("{:X}", hasher.finalize());
        Ok(Value::from(s))
    }
);

define_stdfunction!(
    choose {
        options: Standard::Array
    },
    returns = String,
    docs = {
        category: "Random",
        description: "Returns a random element from a given array",
        ext_description: "Uses a uniform distribution to select a random element from the input array.",
        examples: "
            s = ['a', 'b', 'c']
            assert(
                s contains choose(s)
            )
        "
    },
    handler = (state) {
        let options = required_arg!(state::options).as_a::<Vec<Value>>()?;
        if options.is_empty() {
            return oops!(ArrayEmpty);
        }

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        Ok(options.choose(&mut rng).unwrap().to_string().into())
    }
);

define_stdfunction!(
    rand {
        range: Optional::Range
    },
    returns = Numeric,
    docs = {
        category: "Random",
        description: "Returns a random number within a given range, or between 0 and 1 if no range is specified.",
        ext_description: "
            If no range is specified, the function will return a random number between 0 and 1.
            If a range is specified, the function will return a random number within that range.
        ",
        examples: "
            r = rand(0..10)
            assert(
                r >= 0 && r <= 10
            )
        "
    },
    handler = (state) {
        use rand::Rng;

        if let Some(range) = optional_arg!(state::range) {
            let range = range.as_a::<Range>()?.inner().clone();
            Ok(rand::thread_rng().gen_range(range).into())
        } else {
            Ok(rand::random::<f64>().into())
        }
    }
);
