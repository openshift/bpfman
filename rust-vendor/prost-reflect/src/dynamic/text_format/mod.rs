mod format;
#[cfg(feature = "text-format")]
mod parse;

#[cfg(feature = "text-format")]
pub use self::parse::ParseError;
#[cfg(feature = "text-format")]
use crate::{DynamicMessage, MessageDescriptor};

pub(super) use self::format::Writer;

/// Options to control printing of the protobuf text format.
///
/// Used by [`DynamicMessage::to_text_format_with_options()`].
#[derive(Debug, Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "text-format")))]
pub struct FormatOptions {
    pretty: bool,
    skip_unknown_fields: bool,
    expand_any: bool,
    skip_default_fields: bool,
    print_message_fields_in_index_order: bool,
}

#[cfg(feature = "text-format")]
impl DynamicMessage {
    /// Parse a [`DynamicMessage`] from the given message encoded using the [text format](https://developers.google.com/protocol-buffers/docs/text-format-spec).
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// # let message_descriptor = pool.get_message_by_name("package.MyMessage").unwrap();
    /// let dynamic_message = DynamicMessage::parse_text_format(message_descriptor, "foo: 150").unwrap();
    /// assert_eq!(dynamic_message.get_field_by_name("foo").unwrap().as_ref(), &Value::I32(150));
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "text-format")))]
    pub fn parse_text_format(desc: MessageDescriptor, input: &str) -> Result<Self, ParseError> {
        let mut message = DynamicMessage::new(desc);
        message.merge_text_format(input)?;
        Ok(message)
    }

    /// Merges the given message encoded using the [text format](https://developers.google.com/protocol-buffers/docs/text-format-spec) into this message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// # let message_descriptor = pool.get_message_by_name("package.MyMessage").unwrap();
    /// let mut dynamic_message = DynamicMessage::new(message_descriptor);
    /// dynamic_message.merge_text_format("foo: 150").unwrap();
    /// assert_eq!(dynamic_message.get_field_by_name("foo").unwrap().as_ref(), &Value::I32(150));
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "text-format")))]
    pub fn merge_text_format(&mut self, input: &str) -> Result<(), ParseError> {
        parse::Parser::new(input)
            .parse_message(self)
            .map_err(|kind| ParseError::new(kind, input))
    }

    /// Formats this dynamic message using the protobuf text format, with default options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_types::FileDescriptorSet;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value, text_format::FormatOptions};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// # let message_descriptor = pool.get_message_by_name("package.MyMessage").unwrap();
    /// let dynamic_message = DynamicMessage::decode(message_descriptor, b"\x08\x96\x01\x1a\x02\x10\x42".as_ref()).unwrap();
    /// assert_eq!(dynamic_message.to_text_format(), "foo:150,nested{bar:66}");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "text-format")))]
    pub fn to_text_format(&self) -> String {
        self.to_text_format_with_options(&FormatOptions::new())
    }

    /// Formats this dynamic message using the protobuf text format, with custom options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_types::FileDescriptorSet;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value, text_format::FormatOptions};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// # let message_descriptor = pool.get_message_by_name("package.MyMessage").unwrap();
    /// let dynamic_message = DynamicMessage::decode(message_descriptor, b"\x08\x96\x01\x1a\x02\x10\x42".as_ref()).unwrap();
    /// let options = FormatOptions::new().pretty(true);
    /// assert_eq!(dynamic_message.to_text_format_with_options(&options), "foo: 150\nnested {\n  bar: 66\n}");
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "text-format")))]
    pub fn to_text_format_with_options(&self, options: &FormatOptions) -> String {
        let mut result = String::new();
        format::Writer::new(options.clone(), &mut result)
            .fmt_message(self)
            .expect("writing to string cannot fail");
        result
    }
}

impl FormatOptions {
    /// Creates new instance of [`FormatOptions`] with default options.
    pub fn new() -> Self {
        FormatOptions::default()
    }

    /// Whether to prettify the format output.
    ///
    /// If set to `true`, each field will be printed on a new line, and nested messages will be indented.
    ///
    /// The default value is `false`.
    pub fn pretty(mut self, yes: bool) -> Self {
        self.pretty = yes;
        self
    }

    /// Whether to include unknown fields in the output.
    ///
    /// If set to `false`, unknown fields will be printed. The protobuf format does not include type information,
    /// so the formatter will attempt to infer types.
    ///
    /// The default value is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_types::FileDescriptorSet;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value, text_format::FormatOptions};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// # let message_descriptor = pool.get_message_by_name("google.protobuf.Empty").unwrap();
    /// let dynamic_message = DynamicMessage::decode(message_descriptor, b"\x08\x96\x01\x1a\x02\x10\x42".as_ref()).unwrap();
    /// assert_eq!(dynamic_message.to_text_format(), "");
    /// let options = FormatOptions::new().skip_unknown_fields(false);
    /// assert_eq!(dynamic_message.to_text_format_with_options(&options), "1:150,3{2:66}");
    /// ```
    #[cfg(feature = "text-format")]
    pub fn skip_unknown_fields(mut self, yes: bool) -> Self {
        self.skip_unknown_fields = yes;
        self
    }

    /// Whether to skip fields which have their default value.
    ///
    /// If `true`, any fields for which [`has_field`][DynamicMessage::has_field] returns `false` will
    /// not be included. If `false`, they will be included with their default value.
    ///
    /// The default value is `true`.
    #[cfg(feature = "text-format")]
    pub fn skip_default_fields(mut self, yes: bool) -> Self {
        self.skip_default_fields = yes;
        self
    }

    /// Whether to print message fields in the order they were defined in source code.
    ///
    /// If set to `true`, message fields will be printed in the order they were defined in the source code.
    /// Otherwise, they will be printed in field number order.
    ///
    /// The default value is `false`.
    #[cfg(feature = "text-format")]
    pub fn print_message_fields_in_index_order(mut self, yes: bool) -> Self {
        self.print_message_fields_in_index_order = yes;
        self
    }

    /// Whether to use the expanded form of the `google.protobuf.Any` type.
    ///
    /// If set to `true`, `Any` fields will use an expanded form:
    ///
    /// ```textproto
    /// [type.googleapis.com/package.MyMessage] {
    ///   foo: 150
    /// }
    /// ```
    ///
    /// If set to `false`, the normal text format representation will be used:
    ///
    /// ```textproto
    /// type_url: "type.googleapis.com/package.MyMessage"
    /// value: "\x08\x96\x01"
    /// ```
    ///
    /// The default value is `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use prost::Message;
    /// # use prost_types::FileDescriptorSet;
    /// # use prost_reflect::{DynamicMessage, DescriptorPool, Value, text_format::FormatOptions, bytes::Bytes};
    /// # let pool = DescriptorPool::decode(include_bytes!("../../file_descriptor_set.bin").as_ref()).unwrap();
    /// let message_descriptor = pool.get_message_by_name("google.protobuf.Any").unwrap();
    /// let mut dynamic_message = DynamicMessage::new(message_descriptor);
    /// dynamic_message.set_field_by_name("type_url", Value::String("type.googleapis.com/package.MyMessage".to_owned()));
    /// dynamic_message.set_field_by_name("value", Value::Bytes(Bytes::from_static(b"\x08\x96\x01\x1a\x02\x10\x42".as_ref())));
    ///
    /// assert_eq!(dynamic_message.to_text_format(), "[type.googleapis.com/package.MyMessage]{foo:150,nested{bar:66}}");
    /// let options = FormatOptions::new().expand_any(false);
    /// assert_eq!(dynamic_message.to_text_format_with_options(&options), r#"type_url:"type.googleapis.com/package.MyMessage",value:"\010\226\001\032\002\020B""#);
    /// ```
    #[cfg(feature = "text-format")]
    pub fn expand_any(mut self, yes: bool) -> Self {
        self.expand_any = yes;
        self
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        FormatOptions {
            pretty: false,
            skip_unknown_fields: true,
            expand_any: true,
            skip_default_fields: true,
            print_message_fields_in_index_order: false,
        }
    }
}
