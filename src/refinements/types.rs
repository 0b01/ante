use crate::cache::ModuleCache;
use crate::util::fmap;
use crate::{cache::DefinitionInfoId, lifetimes};
use crate::lexer::token::Token;
use crate::types::{PrimitiveType, Type, TypeInfoId, TypeVariableId};
use crate::refinements::context::RefinementContext;
use crate::util::reinterpret_from_bits;

/// These refinements are boolean expressions that can
/// narrow down the set of values a type accepts. Their
/// semantics are limited to a small logic usually solveable
/// by an SMT solver.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Refinement {
    Variable(DefinitionInfoId),
    Integer(i64),
    Boolean(bool),
    Float(u64),
    Unit,
    Constructor(DefinitionInfoId, Vec<Refinement>),
    PrimitiveCall(Primitive, Vec<Refinement>),
    Uninterpreted(DefinitionInfoId, Vec<Refinement>),
    Forall(DefinitionInfoId, Box<RefinementType>, Box<Refinement>),
}

/// A RefinementType is a Type and a Refinement, t & r
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RefinementType {
    Base(BaseType, Option<(DefinitionInfoId, Refinement)>),
    // TODO: Should Function be a BaseType instead so that this
    // variant can be merged with TypeApplication to be represented
    // as TypeApplication(Function, args) ?
    Function(Vec<NamedType>, Box<RefinementType>),
    TypeApplication(BaseType, Vec<NamedType>),
    ForAll(Vec<TypeVariableId>, Box<RefinementType>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BaseType {
    TypeVariable(TypeVariableId),
    Primitive(PrimitiveType),
    UserDefinedType(TypeInfoId),
    Ref(lifetimes::LifetimeVariableId),
}

pub type NamedType = (DefinitionInfoId, RefinementType);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Primitive {
    Add, Sub, Mul, Div,
    Eq, Neq,
    Lt, Gt, Lte, Gte,
    Implies, And, Or, Not,
}

impl Refinement {
    pub fn find_variables(&self) -> Vec<DefinitionInfoId> {
        use Refinement::*;
        match self {
            Variable(id) => vec![*id],
            Integer(_) => vec![],
            Boolean(_) => vec![],
            Float(_) => vec![],
            Unit => vec![],
            PrimitiveCall(_, args) => args.iter().flat_map(|arg| arg.find_variables()).collect(),
            Constructor(f, args)
            | Uninterpreted(f, args) => {
                let mut vars = vec![*f];
                for arg in args {
                    vars.append(&mut arg.find_variables());
                }
                vars
            }
            Forall(x, _t, e) => {
                let mut vars = vec![*x];
                vars.append(&mut e.find_variables());
                vars
            }
        }
    }

    pub fn substitute(&self, id: DefinitionInfoId, other: &Refinement) -> Refinement {
        use Refinement::*;
        match self {
            Variable(variable_id) if *variable_id == id => other.clone(),
            Variable(nonmatching_id) => Variable(*nonmatching_id),
            Integer(x) => Integer(*x),
            Boolean(b) => Boolean(*b),
            Float(f) => Float(*f),
            Unit => Unit,
            Constructor(f, args) => Constructor(*f, fmap(args, |arg| arg.substitute(id, other))),
            PrimitiveCall(f, args) => PrimitiveCall(*f, fmap(args, |arg| arg.substitute(id, other))),
            Uninterpreted(f, args) => Uninterpreted(*f, fmap(args, |arg| arg.substitute(id, other))),
            Forall(x, t, e) => Forall(*x, t.clone(), Box::new(e.substitute(id, other))),
        }
    }

    pub fn substitute_var(&self, id: DefinitionInfoId, replacement: DefinitionInfoId) -> Refinement {
        self.substitute(id, &Refinement::Variable(replacement))
    }

    pub fn none() -> Refinement {
        Refinement::Boolean(true)
    }

    pub fn implies(self, other: Refinement) -> Refinement {
        Refinement::PrimitiveCall(Primitive::Implies, vec![self, other])
    }

    pub fn and(self, other: Refinement) -> Refinement {
        Refinement::PrimitiveCall(Primitive::And, vec![self, other])
    }

    pub fn eq(self, other: Refinement) -> Refinement {
        Refinement::PrimitiveCall(Primitive::Eq, vec![self, other])
    }

    pub fn not(self) -> Refinement {
        Refinement::PrimitiveCall(Primitive::Not, vec![self])
    }
}

impl std::fmt::Display for Refinement {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Refinement::*;
        match self {
            Variable(v) => write!(f, "{}", v.0),
            Integer(x) => write!(f, "{}", *x),
            Boolean(b) => write!(f, "{}", *b),
            Float(x) => write!(f, "{}", reinterpret_from_bits(*x)),
            Unit => write!(f, "()"),
            PrimitiveCall(c, args) => {
                write!(f, "({}", c)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            Uninterpreted(c, args)
            | Constructor(c, args) => {
                if !args.is_empty() {
                    write!(f, "(")?;
                }
                write!(f, "({}", c.0)?;
                for arg in args {
                    write!(f, " {}", arg)?;
                }
                if !args.is_empty() {
                    write!(f, ")")?;
                }
                Ok(())
            }
            Forall(x, t, e) => write!(f, "forall {}:{:?}. {}", x.0, *t, e),
        }
    }
}

impl Primitive {
    pub fn from_token(token: &Token) -> Option<Primitive> {
        match token {
            Token::Add => Some(Primitive::Add),
            Token::Subtract => Some(Primitive::Sub),
            Token::Multiply => Some(Primitive::Mul),
            Token::Divide => Some(Primitive::Div),
            Token::EqualEqual => Some(Primitive::Eq),
            Token::NotEqual => Some(Primitive::Neq),
            Token::LessThan => Some(Primitive::Lt),
            Token::GreaterThan => Some(Primitive::Gt),
            Token::LessThanOrEqual => Some(Primitive::Lte),
            Token::GreaterThanOrEqual => Some(Primitive::Gte),
            Token::And => Some(Primitive::And),
            Token::Or => Some(Primitive::Or),
            Token::Not => Some(Primitive::Not),
            _ => None,
        }
    }
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Primitive::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Div => write!(f, "/"),
            Eq => write!(f, "=="),
            Neq => write!(f, "!="),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            Lte => write!(f, "<="),
            Gte => write!(f, ">="),
            Implies => write!(f, "=>"),
            And => write!(f, "and"),
            Or => write!(f, "or"),
            Not => write!(f, "not"),
        }
    }
}

impl RefinementType {
    pub fn substitute(&self, id: DefinitionInfoId, replacement: &Refinement) -> RefinementType {
        let replace_id = |x| if x == id { id } else { x };
        use RefinementType::*;
        match self {
            Base(b, refinement) => Base(*b, refinement.as_ref().map(|(value_id, refinement)| {
                (replace_id(*value_id), refinement.substitute(id, &replacement))
            })),
            Function(args, ret) => {
                let args = fmap(args, |(arg_id, arg)| (replace_id(*arg_id), arg.substitute(id, &replacement)));
                let ret = Box::new(ret.substitute(id, &replacement));
                Function(args, ret)
            }
            TypeApplication(base, args) => {
                let args = fmap(args, |(arg_id, arg)| (replace_id(*arg_id), arg.substitute(id, &replacement)));
                TypeApplication(*base, args)
            }
            ForAll(vars, typ) => ForAll(vars.clone(), Box::new(typ.substitute(id, replacement))),
        }
    }

    pub fn substitute_var(&self, id: DefinitionInfoId, replacement: DefinitionInfoId) -> RefinementType {
        self.substitute(id, &Refinement::Variable(replacement))
    }

    /// Return a tuple of (args, ret) if this is a function type,
    /// or panic otherwise.
    pub fn unwrap_function(self) -> (Vec<NamedType>, RefinementType) {
        match self {
            RefinementType::Function(args, ret) => (args, *ret),
            _ => unreachable!(),
        }
    }

    // Is self a subtype of other?
    // Returns a set of constraints that would make this true
    pub fn check_subtype<'c>(&self, other: &RefinementType, _cache: &ModuleCache<'c>) -> Refinement {
        match (self, other) {
            (RefinementType::Base(_, _), RefinementType::Base(_, _)) => todo!(),
            (RefinementType::Function(_, _), RefinementType::Function(_, _)) => todo!(),
            (RefinementType::TypeApplication(_, _), RefinementType::TypeApplication(_, _)) => todo!(),
            (RefinementType::ForAll(_, _), RefinementType::ForAll(_, _)) => todo!(),
            (a, b) => unreachable!("check_subtype found an uncaught type error while checking {:?} and {:?}", a, b),
        }
    }

    pub fn unit() -> RefinementType {
        RefinementType::Base(BaseType::Primitive(PrimitiveType::UnitType), None)
    }

    pub fn bool() -> RefinementType {
        RefinementType::Base(BaseType::Primitive(PrimitiveType::BooleanType), None)
    }

    pub fn bool_refined<'c>(refinement: Refinement, cache: &mut ModuleCache<'c>) -> RefinementType {
        let id = fresh_bool_var(cache);
        RefinementType::Base(BaseType::Primitive(PrimitiveType::BooleanType), Some((id, refinement)))
    }
}

fn fresh_bool_var<'c>(cache: &mut ModuleCache<'c>) -> DefinitionInfoId {
    cache.fresh_internal_var(Type::Primitive(PrimitiveType::BooleanType))
}

// [Ent-Ext]
// G |- forall x:b. p => c
// -----------------------
// G; x:b{x:p} |- c
//
// [Ent-Emp]
// SmtValid(c)
// -----------
// empty |- c
fn entails(context: &RefinementContext, c: &Refinement) -> Refinement {
    context.fold(c.clone(), |_id, p, c| {
        p.clone().implies(c)
    })
}

// [Sub-Base]  
//
// forall (v1:b). p1 => p2[v2 := v1]
// -------------------
// b{v1:p1} <: b{v2:p2}
fn check_subtype_base(_b1: BaseType, v1: DefinitionInfoId, p1: &Refinement, _b2: BaseType, v2: DefinitionInfoId, p2: Refinement) -> Refinement {
    let q = p2.substitute_var(v2, v1);
    Refinement::implies(p1.clone(), q)
}

// [Sub-Fun]  
//
// s2 <: s1    x2:s2 |- t1[x1:=x2] <: t2 
// -------------------------------------
// x1:s1 -> t1 <: x2:s2 -> t2
fn check_subtype_fun<'c>(context: &mut RefinementContext, x1: DefinitionInfoId, s1: &RefinementType,
    t1: &RefinementType, x2: DefinitionInfoId, s2: &RefinementType, t2: &RefinementType, cache: &ModuleCache<'c>) -> Refinement
{
    let r1 = s2.check_subtype(s1, cache);
    context.insert_refinement(x2, s2.clone());
    let t1 = t1.substitute_var(x1, x2);
    let r2 = t1.check_subtype(t2, cache);
    context.remove_refinement(x2);
    r1.and(r2)
}

// [Sub-TCon] 
//
// G,v:int{p} |- q[w:=v]     G |- si <: ti
// -----------------------------------------
// G |- (C s1...)[v|p] <: (C t1...)[w|q]
fn check_subtype_tcon() {

}
