/// Message wrapper that holds buffer positional data and a generic type.
/// This is useful if you want to send a message to another source that
/// something occured at a specific moment, for example:
/// a midi message was triggered at buffer position 42

pub struct Message<T> {
    /// Buffer position the event occured
    pub position: usize,
    /// Generic data specified by the application
    pub data: T,
}

impl<T> Message<T> {
    /// instantiate new `Message<T>`
    /// ```
    /// use screech::Message;
    ///
    /// #[derive(Debug, PartialEq)]
    /// enum MessageType {
    ///     Midi(u8, u8, u8),
    ///     Foo,
    /// }
    ///
    /// let message = Message::new(42, MessageType::Midi(146, 69, 128));
    ///
    /// assert_eq!(message.position, 42);
    /// assert_eq!(message.data, MessageType::Midi(146, 69, 128));
    /// ```
    pub fn new(position: usize, data: T) -> Self {
        Message { position, data }
    }
}
