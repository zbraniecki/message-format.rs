// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::HashMap;
use std::fmt;

use english_cardinal_classifier;
use {Args, Context, Message, MessagePart, PluralCategory, Value};

/// Format a value taking pluralization rules into account.
#[derive(Debug)]
pub struct PluralFormat {
    /// The name of the variable whose value should be formatted.
    variable_name: String,
    classifier: fn(i64) -> PluralCategory,
    literals: HashMap<i64, Message>,
    offset: i64,
    zero: Option<Message>,
    one: Option<Message>,
    two: Option<Message>,
    few: Option<Message>,
    many: Option<Message>,
    other: Message,
}

impl PluralFormat {
    /// Construct a `PluralFormat`.
    pub fn new(variable_name: &str, other: Message) -> Self {
        PluralFormat {
            variable_name: variable_name.to_string(),
            classifier: english_cardinal_classifier,
            literals: HashMap::new(),
            offset: 0,
            zero: None,
            one: None,
            two: None,
            few: None,
            many: None,
            other: other,
        }
    }

    /// Set the `message` to be used for a literal value.
    pub fn literal(&mut self, literal: i64, message: Message) {
        self.literals.insert(literal, message);
    }

    /// Apply an `offset`.
    pub fn offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    /// Set the `message` for `PluralCategory::Zero`.
    pub fn zero(&mut self, message: Message) {
        self.zero = Some(message);
    }

    /// Set the `message` for `PluralCategory::One`.
    pub fn one(&mut self, message: Message) {
        self.one = Some(message);
    }

    /// Set the `message` for `PluralCategory::Two`.
    pub fn two(&mut self, message: Message) {
        self.two = Some(message);
    }

    /// Set the `message` for `PluralCategory::Few`.
    pub fn few(&mut self, message: Message) {
        self.few = Some(message);
    }

    /// Set the `message` for `PluralCategory::Many`.
    pub fn many(&mut self, message: Message) {
        self.many = Some(message);
    }

    /// Given a value adjusted by the `offset`, determine which `Message` to use.
    fn lookup_message(&self, offset_value: i64) -> &Message {
        if let Some(literal) = self.literals.get(&offset_value) {
            literal
        } else {
            let category = (self.classifier)(offset_value);
            match category {
                PluralCategory::Zero => self.zero.as_ref().unwrap_or(&self.other),
                PluralCategory::One => self.one.as_ref().unwrap_or(&self.other),
                PluralCategory::Two => self.two.as_ref().unwrap_or(&self.other),
                PluralCategory::Few => self.few.as_ref().unwrap_or(&self.other),
                PluralCategory::Many => self.many.as_ref().unwrap_or(&self.other),
                PluralCategory::Other => &self.other,
            }
        }
    }
}

impl MessagePart for PluralFormat {
    fn apply_format<'f>(&self,
                        ctx: &Context,
                        stream: &mut fmt::Write,
                        args: Option<&Args<'f>>)
                        -> fmt::Result {
        let arg = args.and_then(|args| args.get(&self.variable_name));
        if let Some(&Value::Number(value)) = arg.map(|a| a.value()) {
            let offset_value = value - self.offset;
            let message = self.lookup_message(offset_value);
            let ctx = Context { placeholder_value: Some(offset_value), ..ctx.clone() };
            try!(message.write_message(&ctx, stream, args));
            Ok(())
        } else {
            Err(fmt::Error {})
        }
    }
}

#[cfg(test)]
mod tests {
    use icu::parse;
    use super::PluralFormat;
    use {arg, Context, MessagePart};

    #[test]
    fn it_works() {
        let ctx = Context::default();
        let mut fmt = PluralFormat::new("count", parse("Other").unwrap());
        fmt.one(parse("One").unwrap());

        let mut output = String::new();
        fmt.apply_format(&ctx, &mut output, Some(&arg("count", 0))).unwrap();
        assert_eq!("Other", output);

        let mut output = String::new();
        fmt.apply_format(&ctx, &mut output, Some(&arg("count", 1))).unwrap();
        assert_eq!("One", output);

        let mut output = String::new();
        fmt.apply_format(&ctx, &mut output, Some(&arg("count", 3))).unwrap();
        assert_eq!("Other", output);
    }
}
