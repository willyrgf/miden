use std::time::Instant;

use super::{
    combine_blocks, parse_code_blocks, AssemblyContext, AssemblyError, CodeBlock, Token,
    TokenStream,
};
use vm_core::{Felt, Operation};

// PROCEDURE
// ================================================================================================

/// Contains metadata and code of a procedure.
pub struct Procedure {
    label: String,
    is_export: bool,
    #[allow(dead_code)]
    num_locals: u32,
    code_root: CodeBlock,
}

impl Procedure {
    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns a root of this procedure's MAST.
    pub fn code_root(&self) -> &CodeBlock {
        &self.code_root
    }

    /// Returns a label of this procedure.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns `true` if this is an exported procedure.
    pub fn is_export(&self) -> bool {
        self.is_export
    }

    // PARSER
    // --------------------------------------------------------------------------------------------

    /// Parses and returns a single procedure from the provided token stream.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The token stream does not contain a procedure header token at the current position.
    /// - Parsing of procedure header token fails (e.g., invalid procedure label).
    /// - The procedure is an exported procedure and `allow_export` is false.
    /// - A procedure with the same label already exists in the provided context.
    /// - Parsing of procedure body fails for any reason.
    /// - The procedure body does not terminate with the `END` token.
    pub fn parse(
        tokens: &mut TokenStream,
        context: &AssemblyContext,
        allow_export: bool,
    ) -> Result<Self, AssemblyError> {
        let proc_start = tokens.pos();

        // read procedure name and consume the procedure header token
        let header = tokens.read().expect("missing procedure header");
        let (label, num_locals, is_export) = header.parse_proc()?;
        if !allow_export && is_export {
            return Err(AssemblyError::prc_export_not_allowed(header, &label));
        }
        if context.contains_proc(&label) {
            return Err(AssemblyError::duplicate_proc_label(header, &label));
        }
        tokens.advance();

        let start = Instant::now();
        // parse procedure body, and handle memory allocation/deallocation of locals if any are declared
        let code_root = parse_proc_blocks(tokens, context, num_locals)?;
        let elapsed = start.elapsed();
        println!(
            "parse_module().tokens.read() loop parse().parse_proc_blocks() elapsed: {:?}",
            elapsed
        );

        let start = Instant::now();
        // consume the 'end' token
        match tokens.read() {
            None => Err(AssemblyError::unmatched_proc(
                tokens.read_at(proc_start).expect("no proc token"),
            )),
            Some(token) => match token.parts()[0] {
                Token::END => token.validate_end(),
                _ => Err(AssemblyError::unmatched_proc(
                    tokens.read_at(proc_start).expect("no proc token"),
                )),
            },
        }?;
        let elapsed = start.elapsed();
        println!(
            "parse_module().tokens.read() loop parse().tokens.read()match() elapsed: {:?}",
            elapsed
        );
        tokens.advance();

        // build and return the procedure
        Ok(Self {
            label,
            is_export,
            num_locals,
            code_root,
        })
    }
}

// HELPER FUNCTIONS
// ================================================================================================

pub fn parse_proc_blocks(
    tokens: &mut TokenStream,
    context: &AssemblyContext,
    num_proc_locals: u32,
) -> Result<CodeBlock, AssemblyError> {
    let start = Instant::now();
    // parse the procedure body
    let body = parse_code_blocks(tokens, context, num_proc_locals)?;
    let elapsed = start.elapsed();
    println!(
        "parse_module().tokens.read() loop parse_proc_blocks().parse_code_blocks() elapsed: {:?}",
        elapsed
    );

    if num_proc_locals == 0 {
        // if no allocation of locals is required, return the procedure body
        return Ok(body);
    }

    let mut blocks = Vec::new();
    let locals_felt = Felt::new(num_proc_locals as u64);

    // allocate procedure locals before the procedure body
    let alloc_ops = vec![Operation::Push(locals_felt), Operation::FmpUpdate];
    blocks.push(CodeBlock::new_span(alloc_ops));

    // add the procedure body code block
    blocks.push(body);

    // deallocate procedure locals after the procedure body
    let dealloc_ops = vec![Operation::Push(-locals_felt), Operation::FmpUpdate];
    blocks.push(CodeBlock::new_span(dealloc_ops));

    let start = Instant::now();
    // combine the local memory alloc/dealloc blocks with the procedure body code block
    let r = combine_blocks(blocks);
    let elapsed = start.elapsed();
    println!(
        "parse_module().tokens.read() loop parse_proc_blocks().combine_blocks() elapsed: {:?}",
        elapsed
    );
    Ok(r)
}
