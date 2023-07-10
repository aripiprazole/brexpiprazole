use std::{cell::RefCell, rc::Rc, sync::Arc};

use asena_ast::{reporter::Reporter, *};

use asena_ast_db::driver::Driver;
use asena_ast_db::scope::{ScopeData, Value, VariantResolution};
use asena_ast_db::vfs::*;

use asena_report::InternalError;

use thiserror::Error;

use crate::error::ResolutionError::*;

pub mod decl;
pub mod error;
pub mod scopes;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use asena_ast_db::{driver::Driver, implementation::*, package::*, vfs::*};
    use asena_grammar::parse_asena_file;
    use asena_prec::{default_prec_table, InfixHandler, PrecReorder};

    use crate::decl::AstResolver;

    #[test]
    fn it_works() {
        let mut prec_table = default_prec_table();

        let db = Driver(Arc::new(AstDatabaseImpl::default()));
        let local_pkg = Package::new(&db, "Local", "0.0.0", Arc::new(Default::default()));
        let global_scope = db.global_scope();

        let file = VfsFile::new(&db, "Test", "./Test.ase".into(), local_pkg);
        VfsFile::new(&db, "Nat", "./Nat.ase".into(), local_pkg);
        VfsFile::new(&db, "IO", "./IO.ase".into(), local_pkg);

        let mut asena_file = parse_asena_file!("../Test.ase");

        global_scope.borrow_mut().import(&db, file.clone(), None);

        db.abstract_syntax_tree(file.clone())
            .arc_walks(InfixHandler {
                prec_table: &mut prec_table,
                reporter: &mut asena_file.reporter,
            })
            .arc_walks(PrecReorder {
                prec_table: &prec_table,
                reporter: &mut asena_file.reporter,
            })
            .arc_walks(AstResolver {
                db,
                file,
                binding_groups: Default::default(),
                enum_declarations: Default::default(),
                class_declarations: Default::default(),
                trait_declarations: Default::default(),
                instance_declarations: Default::default(),
                reporter: &mut asena_file.reporter,
            });

        asena_file.reporter.dump();
    }
}
