use errors::*;
use ast::contract::desc::StructDesc;
use ast::contract::desc::TraitDesc;
use ast::types::AstType;
use genco::java::{self, *};
use genco::IntoTokens;
use genco::Formatter;
use genco::Custom;
use std::fmt::Write;
use std::collections::HashMap;

pub(crate) struct StructGen<'a> {
    pub desc: &'a StructDesc,
    pub pkg: String
}

impl<'a> StructGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut class = Class::new(self.desc.name.clone());
        class.modifiers.push(Modifier::Public);
        class.implements.push(java::imported("java.io", "Serializable"));
        
        for field in self.desc.fields.iter() {
            let mut java_field = match field.ty {
                AstType::Boolean => Field::new(java::BOOLEAN, field.name.clone()),
                AstType::Byte => Field::new(java::BYTE, field.name.clone()),
                AstType::Int => Field::new(java::INTEGER, field.name.clone()),
                AstType::Long => Field::new(java::LONG, field.name.clone()),
                AstType::Float => Field::new(java::FLOAT, field.name.clone()),
                AstType::Double => Field::new(java::DOUBLE, field.name.clone()),
                AstType::String => Field::new(java::imported("java.lang", "String"), field.name.clone()),
                _ => Field::new(java::VOID, field.name.clone())
            };
            java_field.modifiers = vec![Modifier::Public];
            class.fields.push(java_field);
        }

        let mut buf = String::new();
        {
            let mut formatter = Formatter::new(&mut buf);
            let mut extra = java::Extra::new(self.pkg.as_ref());
            java::Java::write_file(class.into_tokens(), &mut formatter, &mut extra, 1)?;
        }

        Ok(buf)
    }
}

// pub(crate) struct CallbackGen<'a> {
//     pub desc: &'a TraitDesc,
//     pub pkg: String
// }

// impl<'a> CallbackGen<'a> {
//     pub(crate) fn gen(&self) -> Result<String> {
//         let mut tokens = toks!("package", " ", self.pkg.clone(), ";");
        
//         let mut interface = Interface::new(self.desc.name.clone());
//         interface.modifiers.push(Modifier::Public);
//         interface.extends = toks!(java::imported("java.io", "Serializable"));

//         for method in self.desc.methods.iter() {
//             let mut m = Method::new(method.name.clone());
//             m.modifiers = vec![Modifier::Abstract];
//             m.returns = 
//             interface.methods.push();
//         }

//         tokens.push(interface.into_tokens());
//         tokens.to_string().chain_err(|| "struct generate failed.")
//     }
// }

// impl From<AstType> for Java<'static> {
//     fn from(item: AstType) -> Self {
//         match item {
//             AstType::Boolean => java::BOOLEAN,
//             AstType::Byte => java::BYTE,
//             AstType::Int => java::INTEGER,
//             AstType::Long => java::LONG,
//             AstType::Float => java::FLOAT,
//             AstType::Double => java::DOUBLE,
//             AstType::String => java::imported("java.lang", "String"),
//             AstType::Vec(base) => {
//                 java::local(name: N)
//             }
//             AstType::Void 
//             | AstType::Callback 
//             | AstType::Struct => java::VOID
//         }
//     }
// }