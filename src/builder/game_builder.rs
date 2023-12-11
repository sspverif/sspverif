use std::collections::HashMap;

use crate::{
    package::{Edge, Export, PackageInstance},
    types::Type,
    util::resolver::{Resolver, SliceResolver},
};

pub enum Error {
    PackageInstanceNameAlreadyInUse(String),
    ConstantNameAlreadyInUse(String),
    PackageInstanceOutgoingEdgeAlreadySpecified,
    ExportAlreadySpecified,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct GameBuilder<'a> {
    pkg_insts: Vec<PackageInstance>,
    consts: Vec<(&'a str, Type)>,
    edges: Vec<Edge>,
    exports: Vec<Export>,
}

impl<'a> GameBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pkg_inst(&mut self, pkg_inst: PackageInstance) -> Result<()> {
        let already_exists = self.pkg_inst_resolver().resolve(&pkg_inst.name).is_some();

        if already_exists {
            return Err(Error::PackageInstanceNameAlreadyInUse(
                pkg_inst.name.clone(),
            ));
        }

        self.pkg_insts.push(pkg_inst);
        Ok(())
    }

    pub fn add_const(&mut self, name: &'a str, tipe: Type) -> Result<()> {
        let already_exists = self.const_resolver().resolve(name).is_some();

        if already_exists {
            return Err(Error::ConstantNameAlreadyInUse(name.to_string()));
        }

        self.consts.push((name, tipe));
        Ok(())
    }

    pub fn add_export(&mut self, export: Export) -> Result<()> {
        let export_already_exists = self
            .exports
            .iter()
            .find(|existing_export| existing_export.1.name == export.1.name)
            .is_some();

        if export_already_exists {
            return Err(Error::ExportAlreadySpecified);
        }

        self.exports.push(export);

        Ok(())
    }

    pub fn add_edge(&mut self, edge: Edge) -> Result<()> {
        let out_edge_already_exists = self
            .edges
            .iter()
            .find(|existing_edge| existing_edge.0 == edge.0 && existing_edge.2.name == edge.2.name)
            .is_some();

        if out_edge_already_exists {
            return Err(Error::PackageInstanceOutgoingEdgeAlreadySpecified);
        }

        self.edges.push(edge);

        Ok(())
    }
}

/*
 *
 *
 * More features to implement:
 *
 * - type params (might already be there due to consts)
 *
 * - loops
 *
 *   this will need a new type that keeps a mut reference to this one and has the loop context.
 *   actually, it might keep a reference to another loop context, they can loop! Or the looping is
 *   handled internally. not sure what the best way to do this is. Options I see:
 *
 *   - internal looping
 *     The loop type can add deeper loop types withing a single struct instance
 *   - polymorphism - enum:
 *     there is an enum that is either a GameBuilder or a GameLoopBuilder, and a GameLoopBuilder
 *     keeps such a reference
 *   - polymorphism - trait:
 *     there is a trait that handles adding instances, edges and exports and it's implemented by
 *     both the game builder and the loop game builder. the loop game builder keeps a reference to
 *     the parent.
 *
 *   i think this might be one of the rare cases where enum-based polymorphism is the right
 *   approach, because with trait-based polymorphism the caller needs to know the concrete type,
 *   which is probably not possible, since we build this dynamically.
 *
 *   -> how do other builder-type types do this? The only thing that comes to mind rn is the format
 *      stuff, and I don't think that is recursive...
 *
 *   I did some experiments and I think the trait-based polymorphism really won't work.
 */

impl<'a> GameBuilder<'a> {
    fn pkg_inst_resolver(&self) -> SliceResolver<PackageInstance> {
        SliceResolver(&self.pkg_insts)
    }

    fn const_resolver(&self) -> SliceResolver<(&'a str, Type)> {
        SliceResolver(&self.consts)
    }
}
