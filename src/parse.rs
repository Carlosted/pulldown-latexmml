mod lex;
mod operator_table;
mod primitives;

use thiserror::Error;

use crate::{
    attribute::{DimensionUnit, Font},
    event::{Content, Event},
};

pub type Dimension = (f32, DimensionUnit);
type Glue = (Dimension, Option<Dimension>, Option<Dimension>);

// FOR NOW:
// - Do not bother about macros, because they will be solvable.
//  Macro expansion could be solvable with `&mut [&'a str]` as input instead of `&mut &'a str`
//  OR
//  It could be solved by using heap allocation for the expansion. If we use heap allocation, we
//  will need to find a way to solve self referencing, or we could just leak a string allocation
//  and drop it when the parser is dropped. Also, this new complete fragment generated by the
//  allocation needs to be matched with what is following. Here is a minimal example:
// ```TeX
// \def\abc{\frac{1}}
//
// $$
// \abc{2}
// $$
// ```
// This should successfully output 1/2
//
// Also:
// ```TeX
//
// \def\abc{\it}
//
// \[
//     \abc 56
// \]
// ```
// This should successfully make the font change.
//
// Either way, we will be fine so lets not worry about it for now.

#[derive(Debug)]
pub enum Instruction<'a> {
    /// Push the event
    Event(Event<'a>),
    /// Parse the substring
    Substring {
        content: &'a str,
        pop_internal_group: bool,
    },
}

#[derive(Debug)]
pub enum GroupType {
    /// The group was initiated by a command which required a subgroup, but should not be apparent
    /// in the rendered output.
    ///
    /// For example, the `\mathbf` command should not output a group, but one is required for the
    /// font state to be changed.
    ///
    /// Semantically, if an `Internal` group is at the top of the stack, then it should be popped
    /// only by an encounter with an empty `Substring` instruction.
    Internal,
    /// The group was initiated by a `{` character.
    Brace,
    /// The group was initiated by a `\begingroup` command.
    BeginGroup,
}

#[derive(Debug)]
pub struct GroupNesting {
    /// The font state of the group.
    font_state: Option<Font>,
    /// How was the group opened?
    group_type: GroupType,
}

#[derive(Debug)]
pub struct Parser<'a> {
    initial_byte_ptr: *const u8,
    /// The next thing that should be parsed or outputed.
    ///
    /// When this is a string/substring, we should parse it. Some commands output
    /// multiple events, so we need to keep track of them and ouput them in the next
    /// iteration before continuing parsing.
    pub(crate) instruction_stack: Vec<Instruction<'a>>,
    /// The initial byte pointer of the input.
    /// The stack representing group nesting.
    pub(crate) group_stack: Vec<GroupNesting>,
}

pub type Result<T> = std::result::Result<T, ParseError>;

// TODO: change invalid char in favor of more expressive errors.
//      - We do not need to know the character, since we know the byte offset.
//      - We need to know _why_ the character is invalid.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid character found in input: {0}")]
    InvalidChar(char),
    #[error(
        "unexpected math `$` (math shift) character - this character is currently unsupported."
    )]
    MathShift,
    #[error("unexpected hash sign `#` character - this character can only be used in macro definitions.")]
    HashSign,
    #[error("unexpected alignment character `&` - this character can only be used in tabular environments (not yet supported).")]
    AlignmentChar,
    #[error("unexpected end of input")]
    EndOfInput,
}

// TODO: make `trim_start` (removing whitespace) calls more systematic.
impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            initial_byte_ptr: input.as_ptr(),
            instruction_stack: Vec::from([
                Instruction::Event(Event::EndGroup),
                Instruction::Substring {
                    content: input,
                    pop_internal_group: true,
                },
                Instruction::Event(Event::BeginGroup),
            ]),
            group_stack: Vec::from([GroupNesting {
                font_state: None,
                group_type: GroupType::Internal,
            }]),
        }
    }

    /// Get the current string we are parsing.
    ///
    /// This returns `None` if the current instruction is not a `Substring`.
    fn current_string<'b>(&'b mut self) -> Result<&'b mut &'a str> {
        let Some(Instruction::Substring {
            content,
            pop_internal_group,
        }) = self.instruction_stack.last()
        else {
            return Err(ParseError::EndOfInput);
        };
        if content.is_empty() {
            if *pop_internal_group {
                let group = self.group_stack.pop();
                assert!(
                    group.is_some_and(|g| matches!(g.group_type, GroupType::Internal)),
                    "(internal error) `internal` group should be at the top of the stack"
                );
            }
            self.instruction_stack.pop();
            self.current_string()
        } else {
            match self.instruction_stack.last_mut() {
                Some(Instruction::Substring { content, .. }) => Ok(content),
                _ => unreachable!(),
            }
        }
    }

    /// Get the current group we are in.
    fn current_group(&self) -> &GroupNesting {
        self.group_stack
            .last()
            .expect("we should always be inside of a group")
    }

    /// Get the current group we are in, mutably.
    fn current_group_mut(&mut self) -> &mut GroupNesting {
        self.group_stack
            .last_mut()
            .expect("we should always be inside of a group")
    }

    /// Return the next event by unwraping it.
    ///
    /// This is an internal function that only works if the `Parser` is currently parsing a string.
    /// If it is so, then we are guaranteed to at least return one event next, the `EndGroup` event.
    fn next_unwrap(&mut self) -> Result<Event<'a>> {
        self.next()
            .expect("we should always have at least one event")
    }

    /// Return the byte index of the current position in the input.
    fn get_byte_index(&self) -> usize {
        // TODO: Here we should check whether the pointer is currently inside a `prelude` or inside
        // of the inputed string.
        // Safety:
        // * Both `self` and `origin` must be either in bounds or one
        //   byte past the end of the same [allocated object].
        //   => this is true, as self never changes the allocation of the `input`.
        //
        // * Both pointers must be *derived from* a pointer to the same object.
        //   (See below for an example.)
        //   => this is true, as `initial_byte_ptr` is derived from `input.as_ptr()`.
        // * The distance between the pointers, in bytes, must be an exact multiple
        //   of the size of `T`.
        //   => this is true, as both pointers are `u8` pointers.
        // * The distance between the pointers, **in bytes**, cannot overflow an `isize`.
        //   => this is true, as the distance is always positive.
        // * The distance being in bounds cannot rely on "wrapping around" the address space.
        //   => this is true, as the distance is always positive.
        todo!()
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.instruction_stack.last_mut() {
            Some(Instruction::Event(_)) => {
                let event = self.instruction_stack.pop().unwrap();
                Some(Ok(match event {
                    Instruction::Event(event) => event,
                    _ => unreachable!(),
                }))
            }
            Some(Instruction::Substring { content, .. }) => {
                if content.is_empty() {
                    self.instruction_stack.pop();
                    return self.next();
                }
                let mut chars = content.chars();
                let next_char = chars.next().expect("the content is not empty");

                Some(match next_char {
                    // TODO: Why are numbers handled here?
                    '.' | '0'..='9' => {
                        let len = content
                            .chars()
                            .skip(1)
                            .take_while(|&c| c.is_ascii_digit() || c == '.')
                            .count()
                            + 1;
                        let (number, rest) = content.split_at(len);
                        *content = rest;
                        Ok(Event::Content(Content::Number {
                            content: number,
                            variant: self.group_stack.last().map(|g| g.font_state).flatten(),
                        }))
                    }
                    '\\' => {
                        *content = &content[1..];
                        let cs = lex::rhs_control_sequence(content);
                        self.handle_primitive(cs)
                    }
                    c => {
                        *content = chars.as_str();
                        self.handle_char_token(c)
                    }
                })
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{Identifier, Operator, Infix};

    use super::*;

    #[test]
    fn test_get_byte_index() {
        todo!()
    }

    // Tests for event generation.
    #[test]
    fn substr_instructions() {
        let parser = Parser::new("\\bar{y}");
        let events = parser.collect::<Result<Vec<_>>>().unwrap();

        println!("{events:?}");

        assert_eq!(
            events,
            vec![
                Event::BeginGroup,
                Event::BeginGroup,
                Event::Content(Content::Identifier(Identifier::Char {
                    content: 'y',
                    variant: None,
                })),
                Event::EndGroup,
                Event::Infix(Infix::Overscript),
                Event::Content(Content::Operator(Operator {
                    content: '‾',
                    stretchy: None,
                    moveable_limits: None,
                    left_space: None,
                    right_space: None,
                    size: None,
                })),
                Event::EndGroup
            ]
        );
    }
}

// Token parsing procedure, as per TeXbook p. 46-47.
//
// This is roughly what the lexer implementation will look like for text mode.
//
// 1. Trim any trailing whitespace from a line.
//
// 2. If '\' (escape character) is encountered, parse the next token.
//  '\n' => _The name is empty_???
//  'is_ascii_alphabetic' => parse until an non ASCII alphabetic, and the name is the token
//  'otherwise' => parse next character, and the name is the symbol.
//
//  Go to SkipBlanks mode if the token is a word or a space symbol.
//  Otherwise, go to MidLine mode.
//
// 3. If `^^` is found:
//  - If the following are two characters of type ASCII lowercase letter or digit,
//  then `^^__` is converted to the correspoding ascii value.
//  - If the following is a single ASCII character, then `^^_` is converted to the corresponding ASCII
//  value with the formula: if `c` is the character, then `c + 64` if `c` if the character has code
//  between 0 and 63, and `c - 64` if the character has code between 64 and 127.
//
//  __Note__: This rule takes precedence over escape character parsing. If such a sequence is found
//  in an escape sequence, it is converted to the corresponding ASCII value.
//
// 4. If the token is a single character, go to MidLine mode.
//
// 5. If the token is an end of line, go to the next line. If nothing was on the line (were in NewLine state), then the
//  `par` token is emitted, meaning that a new paragraph should be started.
//  If the state was MidLine, then the newline is transformed into a space.
//  If the state was SkipBlanks, then the newline is ignored.
//
// 6. Ignore characters from the `Ignore` category.
//
// 7. If the token is a space and the mode is MidLine, the space is transformed into a space token.
//
// 8. If the token is a comment, ignore the rest of the line, and go to the next line.
//
// 9. Go to newlines on the next line.
