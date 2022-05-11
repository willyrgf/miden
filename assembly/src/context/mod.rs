use std::time::Instant;

use super::{CodeBlock, ProcMap, Procedure, MODULE_PATH_DELIM};
use winter_utils::collections::BTreeMap;

// ASSEMBLY CONTEXT
// ================================================================================================

/// Context for a compilation of a given script or module.
///
/// An assembly context contains a set of procedures which can be called from the parsed code.
/// The procedures are divided into local and imported procedures. Local procedures are procedures
/// parsed from the body or a script or a module, while imported procedures are imported from
/// other modules.
///
/// Local procedures are owned by the context, while imported procedures are stored by reference.
pub struct AssemblyContext<'a> {
    local_procs: ProcMap,
    imported_procs: BTreeMap<String, &'a Procedure>,
}

impl<'a> AssemblyContext<'a> {
    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    /// Returns a new empty [AssemblyContext].
    pub fn new() -> Self {
        Self {
            local_procs: BTreeMap::new(),
            imported_procs: BTreeMap::new(),
        }
    }

    // STATE ACCESSORS
    // --------------------------------------------------------------------------------------------

    /// Returns true if a procedure with the specified label exists in this context.
    pub fn contains_proc(&self, label: &str) -> bool {
        self.local_procs.contains_key(label) || self.imported_procs.contains_key(label)
    }

    /// Returns a code root of a procedure for the specified label from this context.
    pub fn get_proc_code(&self, label: &str) -> Option<&CodeBlock> {
        // `expect()`'s are OK here because we first check if a given map contains the key
        if self.imported_procs.contains_key(label) {
            let proc = *self
                .imported_procs
                .get(label)
                .expect("no procedure after contains");
            Some(proc.code_root())
        } else if self.local_procs.contains_key(label) {
            let proc = self
                .local_procs
                .get(label)
                .expect("no procedure after contains");
            Some(proc.code_root())
        } else {
            None
        }
    }

    // STATE MUTATORS
    // --------------------------------------------------------------------------------------------

    /// Adds a local procedure to this context.
    ///
    /// A label for a local procedure is set simply `proc.label`.
    ///
    /// # Panics
    /// Panics if a procedure with the specified label already exists in this context.
    pub fn add_local_proc(&mut self, proc: Procedure) {
        let start = Instant::now();
        let label = proc.label();
        let elapsed = start.elapsed();
        println!(
            "parse_module().tokens.read() loop proc.label() elapsed: {:?}",
            elapsed
        );

        let start = Instant::now();
        assert!(!self.contains_proc(label), "duplicate procedure: {}", label);
        let elapsed = start.elapsed();
        println!(
            "parse_module().tokens.read() loop assert() elapsed: {:?}",
            elapsed
        );

        let start = Instant::now();
        self.local_procs.insert(label.to_string(), proc);
        let elapsed = start.elapsed();
        println!(
            "parse_module().tokens.read() loop local_procs.insert() elapsed: {:?}",
            elapsed
        );
    }

    /// Adds an imported procedure to this context.
    ///
    /// A label for an imported procedure is set to  `prefix::proc.label`.
    ///
    /// # Panics
    /// Panics if a procedure with the specified label already exists in this context.
    pub fn add_imported_proc(&mut self, prefix: &str, proc: &'a Procedure) {
        let label = format!("{}{}{}", prefix, MODULE_PATH_DELIM, proc.label());
        assert!(
            !self.contains_proc(&label),
            "duplicate procedure: {}",
            label
        );
        self.imported_procs.insert(label, proc);
    }

    /// Extracts local procedures from this context.
    pub fn into_local_procs(self) -> ProcMap {
        self.local_procs
    }
}
