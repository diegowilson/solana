use crate::parse_token::parse_token;
use inflector::Inflector;
use serde_json::Value;
use solana_account_decoder::parse_token::spl_token_id_v1_0;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use std::{
    collections::HashMap,
    str::{from_utf8, FromStr},
};
use thiserror::Error;

lazy_static! {
    static ref MEMO_PROGRAM_ID: Pubkey =
        Pubkey::from_str(&spl_memo_v1_0::id().to_string()).unwrap();
    static ref TOKEN_PROGRAM_ID: Pubkey = spl_token_id_v1_0();
    static ref PARSABLE_PROGRAM_IDS: HashMap<Pubkey, ParsableProgram> = {
        let mut m = HashMap::new();
        m.insert(*MEMO_PROGRAM_ID, ParsableProgram::SplMemo);
        m.insert(*TOKEN_PROGRAM_ID, ParsableProgram::SplToken);
        m
    };
}

#[derive(Error, Debug)]
pub enum ParseInstructionError {
    #[error("{0:?} instruction not parsable")]
    InstructionNotParsable(ParsableProgram),

    #[error("{0:?} instruction key mismatch")]
    InstructionKeyMismatch(ParsableProgram),

    #[error("Program not parsable")]
    ProgramNotParsable,

    #[error("Internal error, please report")]
    SerdeJsonError(#[from] serde_json::error::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedInstruction {
    pub program: String,
    pub program_id: String,
    pub parsed: Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedInstructionEnum {
    #[serde(rename = "type")]
    pub instruction_type: String,
    pub info: Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParsableProgram {
    SplMemo,
    SplToken,
}

pub fn parse(
    program_id: &Pubkey,
    instruction: &CompiledInstruction,
    account_keys: &[Pubkey],
) -> Result<ParsedInstruction, ParseInstructionError> {
    let program_name = PARSABLE_PROGRAM_IDS
        .get(program_id)
        .ok_or_else(|| ParseInstructionError::ProgramNotParsable)?;
    let parsed_json = match program_name {
        ParsableProgram::SplMemo => parse_memo(instruction),
        ParsableProgram::SplToken => serde_json::to_value(parse_token(instruction, account_keys)?)?,
    };
    Ok(ParsedInstruction {
        program: format!("{:?}", program_name).to_kebab_case(),
        program_id: program_id.to_string(),
        parsed: parsed_json,
    })
}

fn parse_memo(instruction: &CompiledInstruction) -> Value {
    Value::String(from_utf8(&instruction.data).unwrap().to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse() {
        let memo_instruction = CompiledInstruction {
            program_id_index: 0,
            accounts: vec![],
            data: vec![240, 159, 166, 150],
        };
        assert_eq!(
            parse(&MEMO_PROGRAM_ID, &memo_instruction, &[]).unwrap(),
            ParsedInstruction {
                program: "spl-memo".to_string(),
                program_id: MEMO_PROGRAM_ID.to_string(),
                parsed: json!("🦖"),
            }
        );

        let non_parsable_program_id = Pubkey::new(&[1; 32]);
        assert!(parse(&non_parsable_program_id, &memo_instruction, &[]).is_err());
    }
}