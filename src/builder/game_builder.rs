use std::{collections::HashMap, marker::PhantomData};

use crate::{
    expressions::Expression,
    identifier::Identifier,
    package::{Composition, Edge, Export, MultiInstanceEdge, MultiInstanceExport, PackageInstance},
    types::Type,
    util::resolver::{Resolver, SliceResolver},
};

pub enum Error {
    UndefinedConstant(String),
    PackageInstanceNameAlreadyInUse(String),
    ConstantNameAlreadyInUse(String),
    PackageInstanceOutgoingEdgeAlreadySpecified,
    ExportAlreadySpecified,
    UndefinedPackageInstanceName(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct GameBuilder<'a> {
    pkg_insts: Vec<PackageInstance>,
    consts: Vec<(String, Type)>,
    edges: Vec<Edge>,
    exports: Vec<Export>,
    multi_instance_edges: Vec<MultiInstanceEdge>,
    phantom: [&'a (); 0],
}

impl<'a> GameBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_pkg_inst(&mut self, pkg_inst: PackageInstance) -> Result<()> {
        let already_exists = self
            .pkg_inst_resolver()
            .resolve_value(&pkg_inst.name)
            .is_some();

        if already_exists {
            return Err(Error::PackageInstanceNameAlreadyInUse(
                pkg_inst.name.clone(),
            ));
        }

        self.pkg_insts.push(pkg_inst);
        Ok(())
    }

    pub fn add_const(&mut self, name: &'a str, tipe: Type) -> Result<()> {
        let already_exists = self.const_resolver().resolve_value(name).is_some();

        if already_exists {
            return Err(Error::ConstantNameAlreadyInUse(name.to_string()));
        }

        self.consts.push((name.to_string(), tipe));
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

    pub fn build_for_loop(
        &mut self,
        loop_var_name: &str,
        range_start: Expression,
        range_end: Expression,
    ) -> ForLoopBuilder {
        todo!()
    }

    pub fn finish(self) -> Composition {
        todo!()
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

pub enum GameBlockBuilder<'a> {
    GameBuilder(GameBuilder<'a>),
    ForLoopBuilder(ForLoopBuilder<'a>),
}

impl<'a> GameBlockBuilder<'a> {
    pub fn is_identifier_defined(&self, ident: &Identifier) -> bool {
        match self {
            GameBlockBuilder::GameBuilder(builder) => {
                let res = SliceResolver(&builder.consts);
                res.resolve_value(&ident.ident()).is_some()
            }
            GameBlockBuilder::ForLoopBuilder(builder) => builder.is_identifier_defined(ident),
        }
    }

    pub fn resolve_pkg_instance_by_name(&self, name: &str) -> Option<usize> {
        match self {
            GameBlockBuilder::GameBuilder(builder) => {
                SliceResolver(&builder.pkg_insts).resolve_index(name)
            }
            GameBlockBuilder::ForLoopBuilder(builder) => {
                builder.parent_builder.resolve_pkg_instance_by_name(name)
            }
        }
    }

    fn add_multi_instance_edge(&mut self, edge: MultiInstanceEdge) {
        match self {
            GameBlockBuilder::GameBuilder(builder) => {
                builder.multi_instance_edges.push(edge);
            }
            GameBlockBuilder::ForLoopBuilder(builder) => {
                builder.add_multi_instance_edge(edge);
            }
        }
    }

    fn get_pkg_inst(&self, pkg_offs: usize) -> &PackageInstance {
        match self {
            GameBlockBuilder::GameBuilder(builder) => &builder.pkg_insts[pkg_offs],
            GameBlockBuilder::ForLoopBuilder(builder) => builder.get_pkg_inst(pkg_offs),
        }
    }
}

pub struct ForLoopBuilder<'a> {
    parent_builder: &'a mut GameBlockBuilder<'a>,
    loop_var_name: &'a str,
    range_start: Expression,
    range_end: Expression,
}

impl<'a> ForLoopBuilder<'a> {
    pub fn is_identifier_defined(&self, ident: &Identifier) -> bool {
        self.loop_var_name == ident.ident() || self.parent_builder.is_identifier_defined(ident)
    }
    pub fn multi_instance_edge(&'a mut self) -> MultiInstanceEdgeBuilder<'a> {
        MultiInstanceEdgeBuilder {
            parent_builder: self,
            src_pkginst_idx: None,
            dst_pkginst_idx: None,
            src_indices: vec![],
            dst_indices: vec![],
            oracle_name: None,
        }
    }

    fn add_multi_instance_edge(&mut self, mut edge: MultiInstanceEdge) -> &mut Self {
        let own_loopvar = (
            self.loop_var_name.to_string(),
            self.range_start.clone(),
            self.range_end.clone(),
        );
        edge.loopvars.push(own_loopvar);
        self.parent_builder.add_multi_instance_edge(edge);
        self
    }

    pub fn add_multi_instance_export(&mut self, export: MultiInstanceExport) -> &mut Self {
        todo!();
        self
    }

    pub fn finish(self) -> &'a mut GameBlockBuilder<'a> {
        self.parent_builder
    }

    pub fn get_pkg_inst(&self, pkg_offs: usize) -> &PackageInstance {
        self.parent_builder.get_pkg_inst(pkg_offs)
    }
}

pub struct MultiInstanceEdgeBuilder<'a> {
    parent_builder: &'a mut ForLoopBuilder<'a>,
    src_pkginst_idx: Option<usize>,
    dst_pkginst_idx: Option<usize>,
    src_indices: Vec<Identifier>,
    dst_indices: Vec<(Identifier, Expression)>,
    oracle_name: Option<String>,
}

impl<'a> MultiInstanceEdgeBuilder<'a> {
    pub fn src_instance_name(&mut self, name: &str) -> Result<&mut Self> {
        assert!(self.src_pkginst_idx.is_none());

        self.src_pkginst_idx = self
            .parent_builder
            .parent_builder
            .resolve_pkg_instance_by_name(name);
        if self.src_pkginst_idx.is_none() {
            return Err(Error::UndefinedPackageInstanceName(name.to_string()));
        }

        Ok(self)
    }

    pub fn add_src_index(&mut self, ident: Identifier) -> Result<&mut Self> {
        if !self.parent_builder.is_identifier_defined(&ident) {
            return Err(Error::UndefinedConstant(ident.ident()));
        }

        self.src_indices.push(ident);
        Ok(self)
    }

    pub fn oracle_name(&mut self, name: &str) -> Result<&mut Self> {
        self.oracle_name = Some(name.to_string());
        Ok(self)
    }

    pub fn dst_instance_name(&mut self, name: &str) -> Result<&mut Self> {
        assert!(self.dst_pkginst_idx.is_none());

        self.dst_pkginst_idx = self
            .parent_builder
            .parent_builder
            .resolve_pkg_instance_by_name(name);
        if self.dst_pkginst_idx.is_none() {
            return Err(Error::UndefinedPackageInstanceName(name.to_string()));
        }

        Ok(self)
    }

    pub fn add_dst_index(&mut self, ident: Identifier, expr: Expression) -> Result<&mut Self> {
        if !self.parent_builder.is_identifier_defined(&ident) {
            return Err(Error::UndefinedConstant(ident.ident()));
        }

        // TODO: still need to verify that expression is evaluatable
        todo!();

        self.dst_indices.push((ident, expr));
        Ok(self)
    }

    pub fn finish(self) -> &'a mut ForLoopBuilder<'a> {
        let dst_pkg_inst: &PackageInstance = self
            .parent_builder
            .get_pkg_inst(self.dst_pkginst_idx.unwrap());
        let oracle_resolver = SliceResolver(&dst_pkg_inst.pkg.oracles);
        let oracle_sig = oracle_resolver
            .resolve_value(&self.oracle_name.unwrap())
            .map(|odef| odef.sig.clone())
            .unwrap();

        let edge = MultiInstanceEdge {
            loopvars: vec![],
            source_pkgidx: self.src_pkginst_idx.unwrap(),
            source_instance_idx: self.src_indices,
            dest_pkgidx: self.dst_pkginst_idx.unwrap(),
            dest_instance_idx: self.dst_indices,
            oracle_sig,
        };

        self.parent_builder.add_multi_instance_edge(edge);
        self.parent_builder
    }
}
/*
 *
 * composition {
 *
 *   blarb foo=bar {
 *
 *      foo ... {
 *
 *      }
 *
 *   }
 *
 *   for ... {
 *      <--- w
 *      for .... {
 *         <--- d
 *
 *         compose {
 *           Foo[d, w]: {
 *              with index [d=d-1, w=w] Get: GameInst
 *           }
 *         }
 *      }
 *   }
 * }
 */

impl<'a> GameBuilder<'a> {
    fn pkg_inst_resolver(&self) -> SliceResolver<PackageInstance> {
        SliceResolver(&self.pkg_insts)
    }

    fn const_resolver(&self) -> SliceResolver<(String, Type)> {
        SliceResolver(&self.consts)
    }
}
