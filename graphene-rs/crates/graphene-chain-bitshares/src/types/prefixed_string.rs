macro_rules! define_prefixed_string_type {
    (
        $(#[$type_meta:meta])*
        type: $type_name:ident,
        error: $error_name:ident,
        prefix: $prefix:literal,
        empty_doc: $empty_doc:literal,
        wrong_prefix_doc: $wrong_prefix_doc:literal,
        empty_message: $empty_message:literal,
        wrong_prefix_message: $wrong_prefix_message:literal $(,)?
    ) => {
        use core::fmt;
        use core::str::FromStr;

        const PREFIX: &str = $prefix;

        $(#[$type_meta])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $type_name(String);

        impl $type_name {
            /// Parses and validates a chain-prefixed string value.
            pub fn new(value: impl Into<String>) -> Result<Self, $error_name> {
                let value = value.into();
                validate_prefixed_value(&value)?;
                Ok(Self(value))
            }

            /// Returns the value as its original chain-prefixed string.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $type_name {
            type Err = $error_name;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error as _;

                let value = <&str>::deserialize(deserializer)?;
                value.parse::<Self>().map_err(D::Error::custom)
            }
        }

        /// Error returned when parsing a chain-prefixed string value fails.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $error_name {
            #[doc = $empty_doc]
            Empty,
            #[doc = $wrong_prefix_doc]
            WrongPrefix,
        }

        impl fmt::Display for $error_name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Empty => f.write_str($empty_message),
                    Self::WrongPrefix => f.write_str($wrong_prefix_message),
                }
            }
        }

        fn validate_prefixed_value(value: &str) -> Result<(), $error_name> {
            if value.is_empty() {
                return Err($error_name::Empty);
            }

            if !value.starts_with(PREFIX) || value.len() == PREFIX.len() {
                return Err($error_name::WrongPrefix);
            }

            Ok(())
        }
    };
}

pub(crate) use define_prefixed_string_type;
