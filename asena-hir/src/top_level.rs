use asena_hir_derive::*;

use crate::{attr::HirAttrId, expr::HirExprId, hir_type::HirTypeId, pattern::HirPatternId, *};

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
#[hir_node(HirTopLevel)]
#[hir_debug]
pub struct HirTopLevelEnum {
    pub signature: data::HirSignature,
    pub variants: im::HashMap<NameId, data::HirVariant>,
    pub groups: Vec<HirBindingGroup>,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
#[hir_node(HirTopLevel)]
#[hir_debug]
pub struct HirTopLevelStruct {
    pub signature: data::HirSignature,
    pub fields: im::HashMap<NameId, HirTypeId>,
    pub groups: Vec<HirBindingGroup>,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
#[hir_node(HirTopLevel)]
#[hir_debug]
pub struct HirTopLevelTrait {
    pub signature: data::HirSignature,
    pub groups: Vec<HirBindingGroup>,
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
#[hir_node(HirTopLevel)]
#[hir_debug]
pub struct HirBindingGroup {
    pub signature: data::HirSignature,
    pub declarations: Vec<data::HirDeclaration>,
}

#[derive(Default, Hash, Clone, Debug, PartialEq, Eq)]
#[hir_kind(HirTopLevel)]
pub enum HirTopLevelKind {
    #[default]
    Error,
    HirTopLevelEnum(HirTopLevelEnum),
    HirTopLevelStruct(HirTopLevelStruct),
    HirTopLevelTrait(HirTopLevelTrait),
    HirBindingGroup(HirBindingGroup),
}

#[hir_struct(HirVisitor)]
#[derive(Default, Hash, Clone, Debug, PartialEq, Eq)]
pub struct HirTopLevel {
    pub kind: HirTopLevelKind,
    pub attributes: Vec<HirAttrId>,
    pub docs: Vec<data::HirDoc>,
}

/// Data structures module split into its own module to better disposition, as
/// it is a bit large, and it's used as extension to [`HirTopLevel`].
pub mod data {
    use super::*;

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub struct HirDoc {
        pub text: String,
    }

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub struct HirParameterData {
        pub name: NameId,
        pub parameter_type: Option<HirTypeId>,
    }

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub enum HirParameterKind {
        This, // The self parameter
        Explicit(HirParameterData),
        Implicit(HirParameterData),
    }

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub struct HirSignature {
        pub name: NameId,
        pub parameters: im::HashMap<NameId, HirParameterKind>,
        pub return_type: Option<HirTypeId>,
    }

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub struct HirDeclaration {
        pub patterns: Vec<HirPatternId>,
        pub value: HirExprId,
    }

    #[derive(Hash, Clone, Debug, PartialEq, Eq)]
    #[hir_debug]
    pub struct HirVariant {
        pub name: NameId,
        pub variant_type: HirTypeId,
    }
}
