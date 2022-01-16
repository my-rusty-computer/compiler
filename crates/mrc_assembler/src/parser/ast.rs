#![allow(dead_code)]
//! Parsers for AST nodes.

use crate::{
    ast,
    parser::base::{identifier, number},
    ParseResult, Span,
};
use mrc_instruction::{AddressingMode, OperandSize, Operation, Segment, SizedRegister};
use nom::{
    branch::alt,
    bytes::complete::take,
    character::complete::{char, multispace0, space0, space1},
    combinator::{map, map_res, opt, recognize},
    error::ParseError,
    multi::many0,
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use std::str::FromStr;

fn value_or_label(input: Span) -> ParseResult<ast::ValueOrLabel> {
    alt((
        map(
            delimited(char('\''), take(1usize), char('\'')),
            |res: Span| {
                let c = res.chars().next().unwrap();
                ast::ValueOrLabel::Value(c as i32)
            },
        ),
        map(number, ast::ValueOrLabel::Value),
        map(identifier, |res| {
            ast::ValueOrLabel::Label(res.fragment().to_string())
        }),
    ))(input)
}

fn label(input: Span) -> ParseResult<String> {
    map(
        terminated(terminated(identifier, space0), char(':')),
        |res| res.fragment().to_string(),
    )(input)
}

fn register_operand(input: Span) -> ParseResult<ast::Operand> {
    map_res(identifier, |res| {
        match SizedRegister::from_str(res.fragment()) {
            Ok(sized_register) => Ok(ast::Operand::Register(sized_register)),
            Err(_) => Err(nom::Err::Error(nom::error::Error::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            ))),
        }
    })(input)
}

fn segment_operand(input: Span) -> ParseResult<ast::Operand> {
    let (input, segment) = identifier(input)?;

    match Segment::from_str(segment.fragment()) {
        Ok(segment) => Ok((input, ast::Operand::Segment(segment))),
        Err(_) => Err(nom::Err::Error(nom::error::Error::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))),
    }
}

fn immediate_operand(input: Span) -> ParseResult<ast::Operand> {
    map(value_or_label, ast::Operand::Immediate)(input)
}

fn operand_size(input: Span) -> ParseResult<OperandSize> {
    map_res(identifier, |res| match *res.fragment() {
        "byte" => Ok(OperandSize::Byte),
        "word" => Ok(OperandSize::Word),
        _ => Err(nom::Err::Error(nom::error::Error::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))),
    })(input)
}

enum DirectOrIndirect {
    Direct(ast::ValueOrLabel),
    Indirect(AddressingMode),
}

fn direct_or_indirect(input: Span) -> ParseResult<DirectOrIndirect> {
    alt((
        map(
            map_res(
                recognize(pair(
                    identifier,
                    opt(preceded(delimited(space0, char('+'), space0), identifier)),
                )),
                |res| AddressingMode::from_str(res.fragment()),
            ),
            DirectOrIndirect::Indirect,
        ),
        map(value_or_label, DirectOrIndirect::Direct),
    ))(input)
}

fn direct_or_indirect_operand(input: Span) -> ParseResult<ast::Operand> {
    map(
        tuple((
            opt(terminated(operand_size, space1)),
            opt(terminated(
                terminated(
                    map_res(identifier, |res| Segment::from_str(res.fragment())),
                    char(':'),
                ),
                space0,
            )),
            delimited(
                char('['),
                delimited(space0, direct_or_indirect, space0),
                char(']'),
            ),
        )),
        |(maybe_operand_size, segment_override, direct_or_indirect)| match direct_or_indirect {
            DirectOrIndirect::Direct(value_or_label) => {
                ast::Operand::Direct(value_or_label, maybe_operand_size, segment_override)
            }
            DirectOrIndirect::Indirect(addressing_mode) => {
                ast::Operand::Indirect(addressing_mode, maybe_operand_size, segment_override)
            }
        },
    )(input)
}

fn operand(input: Span) -> ParseResult<ast::Operand> {
    alt((
        direct_or_indirect_operand,
        register_operand,
        segment_operand,
        immediate_operand,
    ))(input)
}

fn operand_set(input: Span) -> ParseResult<ast::OperandSet> {
    map_res(
        tuple((
            opt(operand),
            opt(tuple((delimited(space0, char(','), space0), operand))),
        )),
        |res| match res {
            (Some(destination), Some((_, source))) => {
                Ok(ast::OperandSet::DestinationAndSource(destination, source))
            }
            (Some(destination), None) => Ok(ast::OperandSet::Destination(destination)),
            (None, None) => Ok(ast::OperandSet::None),
            _ => Err(nom::Err::Error(nom::error::Error::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            ))),
        },
    )(input)
}

fn instruction(input: Span) -> ParseResult<ast::Instruction> {
    fn operation(input: Span) -> ParseResult<Operation> {
        map_res(identifier, |res| {
            Operation::from_str(res.fragment()).map_err(|_| {
                nom::Err::Error(nom::error::Error::from_error_kind(
                    input,
                    nom::error::ErrorKind::Eof,
                ))
            })
        })(input)
    }

    map(
        tuple((operation, opt(space1), operand_set)),
        |(operation, _, operand_set)| ast::Instruction::new(operation, operand_set),
    )(input)
}

fn line(input: Span) -> ParseResult<ast::Line> {
    alt((
        map(terminated(label, opt(multispace0)), |label| {
            ast::Line::Label(label)
        }),
        map(terminated(instruction, opt(multispace0)), |instruction| {
            ast::Line::Instruction(instruction)
        }),
    ))(input)
}

pub fn program(input: Span) -> ParseResult<Vec<ast::Line>> {
    preceded(multispace0, many0(line))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mrc_instruction::{AddressingMode, OperandSize, Register};

    #[test]
    fn parse_value_or_label() {
        assert_eq!(
            value_or_label(Span::new("0xB800")).unwrap().1,
            ast::ValueOrLabel::Value(0xB800)
        );
        assert_eq!(
            value_or_label(Span::new("label")).unwrap().1,
            ast::ValueOrLabel::Label("label".to_string())
        );
    }

    #[test]
    fn parse_direct_or_indirect_operand() {
        assert_eq!(
            direct_or_indirect_operand(Span::new("[10]")).unwrap().1,
            ast::Operand::Direct(ast::ValueOrLabel::Value(10), None, None)
        );

        assert_eq!(
            direct_or_indirect_operand(Span::new("byte [10]"))
                .unwrap()
                .1,
            ast::Operand::Direct(ast::ValueOrLabel::Value(10), Some(OperandSize::Byte), None)
        );

        assert_eq!(
            direct_or_indirect_operand(Span::new("byte cs:[10]"))
                .unwrap()
                .1,
            ast::Operand::Direct(
                ast::ValueOrLabel::Value(10),
                Some(OperandSize::Byte),
                Some(Segment::CS)
            )
        );

        assert_eq!(
            direct_or_indirect_operand(Span::new("byte [test]"))
                .unwrap()
                .1,
            ast::Operand::Direct(
                ast::ValueOrLabel::Label("test".to_string()),
                Some(OperandSize::Byte),
                None
            )
        );

        assert_eq!(
            direct_or_indirect_operand(Span::new("[si]")).unwrap().1,
            ast::Operand::Indirect(AddressingMode::Si, None, None)
        );
        assert_eq!(
            direct_or_indirect_operand(Span::new("byte [bx+si]"))
                .unwrap()
                .1,
            ast::Operand::Indirect(AddressingMode::BxSi, Some(OperandSize::Byte), None)
        );
    }

    #[test]
    fn parse_operand() {
        // Immediate
        assert_eq!(
            operand(Span::new("label")).unwrap().1,
            ast::Operand::Immediate(ast::ValueOrLabel::Label("label".to_string()))
        );

        // Register
        assert_eq!(
            operand(Span::new("ax")).unwrap().1,
            ast::Operand::Register(SizedRegister(Register::AlAx, OperandSize::Word))
        );

        // Segment
        assert_eq!(
            operand(Span::new("cs")).unwrap().1,
            ast::Operand::Segment(Segment::CS)
        );

        // Direct
        assert_eq!(
            operand(Span::new("[0xB800]")).unwrap().1,
            ast::Operand::Direct(ast::ValueOrLabel::Value(0xB800), None, None)
        );

        // Direct
        assert_eq!(
            operand(Span::new("word [label]")).unwrap().1,
            ast::Operand::Direct(
                ast::ValueOrLabel::Label("label".to_string()),
                Some(OperandSize::Word),
                None
            )
        );

        // Indirect
        assert_eq!(
            operand(Span::new("[bx+si]")).unwrap().1,
            ast::Operand::Indirect(AddressingMode::BxSi, None, None),
        );
    }

    #[test]
    fn parse_operand_set() {
        assert_eq!(
            operand_set(Span::new("ax, [bx+si]")).unwrap().1,
            ast::OperandSet::DestinationAndSource(
                ast::Operand::Register(SizedRegister(Register::AlAx, OperandSize::Word)),
                ast::Operand::Indirect(AddressingMode::BxSi, None, None),
            )
        );

        assert_eq!(
            operand_set(Span::new("ax")).unwrap().1,
            ast::OperandSet::Destination(ast::Operand::Register(SizedRegister(
                Register::AlAx,
                OperandSize::Word
            )))
        );

        assert_eq!(operand_set(Span::new("")).unwrap().1, ast::OperandSet::None);
    }

    #[test]
    fn parse_instruction() {
        assert_eq!(
            instruction(Span::new("mov ax, bx")).unwrap().1,
            ast::Instruction::new(
                Operation::MOV,
                ast::OperandSet::DestinationAndSource(
                    ast::Operand::Register(SizedRegister(Register::AlAx, OperandSize::Word)),
                    ast::Operand::Register(SizedRegister(Register::BlBx, OperandSize::Word))
                )
            )
        );
    }

    #[test]
    fn parse_label() {
        assert_eq!(label(Span::new("label:")).unwrap().1, "label".to_string());
    }

    #[test]
    fn parse_line() {
        assert_eq!(
            line(Span::new("label:\n")).unwrap().1,
            ast::Line::Label(String::from("label"))
        );

        assert_eq!(
            line(Span::new("mov ax, bx:\n")).unwrap().1,
            ast::Line::Instruction(ast::Instruction::new(
                Operation::MOV,
                ast::OperandSet::DestinationAndSource(
                    ast::Operand::Register(SizedRegister(Register::AlAx, OperandSize::Word)),
                    ast::Operand::Register(SizedRegister(Register::BlBx, OperandSize::Word)),
                )
            ))
        );
    }

    #[test]
    fn parse_program() {
        let source = r"
                push    bx
                push    cx
                push    dx
                push    si
                push    di

            loop_strcmp_loop1:
                mov     al, byte cs:[si]
                inc     si
                cmp     al, bl
                jbe     loop_strcmp_great
                cmp     al, bl
                jl      loop_strcmp_less
                cmp     al, '$'
                je      loop_strcmp_quit
                cmp     bl, '$'
                je      loop_strcmp_quit
                jmp     loop_strcmp_loop1

            loop_strcmp_quit:
                mov     al, 1
                jmp     done

            loop_strcmp_great:
                mov     al, 2
                jmp     done

            loop_strcmp_less:
                mov     al, 0
                jmp     done

            done:
                pop     di
                pop     si
                pop     dx
                pop     cx
                pop     bx
                ret
            ";

        let p = program(Span::new(source));
        println!("{:#?}", p);
    }
}
