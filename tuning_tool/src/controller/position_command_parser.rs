use std::collections::VecDeque;

use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{all_consuming, opt},
    error::Error,
    multi::separated_list0,
    number::complete::float,
    sequence::{delimited, preceded, terminated},
};

use protocol::PositionCommand;

pub struct CommandParser {
    command_queue: VecDeque<PositionCommand>,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            command_queue: VecDeque::new(),
        }
    }

    pub fn reset(&mut self) {
        self.command_queue.clear();
    }

    pub fn get_command(&mut self) -> Option<PositionCommand> {
        self.command_queue.pop_front()
    }

    pub fn have_data(&self) -> bool {
        !self.command_queue.is_empty()
    }

    pub fn parse<'a>(&mut self, input: &'a str) -> Result<(), nom::Err<Error<&'a str>>> {
        // Improvement, this might be time-consuming if user passes lots of commands,
        // maybe we can parse the commands in a thread without blocking users.
        let (_, result) = all_consuming(Self::parse_position_commands).parse(input)?;
        self.command_queue = VecDeque::from(result);
        Ok(())
    }

    fn parse_position_commands(input: &str) -> IResult<&str, Vec<PositionCommand>> {
        let sep = || delimited(multispace0, tag(";"), multispace0);
        let (input, commands) =
            terminated(separated_list0(sep(), Self::parse_floats), opt(sep())).parse(input)?;
        Ok((input, commands))
    }

    fn parse_floats(input: &str) -> IResult<&str, PositionCommand> {
        // Consume '(' with optional surrounding whitespace, and parse first float A in '(A'
        let (input, _) = delimited(multispace0, tag("("), multispace0).parse(input)?;
        let (input, displacement) = float(input)?;

        // Consume optional surrounding whitespace and comma, and parse second float B in '(A, B'
        let (input, _) = delimited(multispace0, tag(","), multispace0).parse(input)?;
        let (input, vel_max) = float(input)?;

        // The third float C is optional '(A, B, C'), and it could be:
        // * '(A, B, C)'
        // * '(A, B)'
        // * '(A, B,)'
        // so we need to optionally parse a comma and a third float.
        let (input, vel_end_opt) = preceded(
            delimited(multispace0, opt(tag(",")), multispace0),
            opt(float),
        )
        .parse(input)?;

        // Consume the closing ')'.
        let (input, _) = delimited(multispace0, tag(")"), multispace0).parse(input)?;

        // If user doesn't specify third float(vel_end), we will treat it as 0 to perform 'buffered mode' motion
        let vel_end = vel_end_opt.unwrap_or(0.0);
        Ok((
            input,
            PositionCommand {
                displacement,
                vel_max,
                vel_end,
            },
        ))
    }
}
