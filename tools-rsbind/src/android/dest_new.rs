use errors::*;
use ast::contract::desc::StructDesc;
use ast::contract::desc::TraitDesc;
use ast::types::AstType;
use genco::java::{self, *};
use genco::IntoTokens;
use genco::Formatter;
use genco::Custom;
use std::fmt::Write;

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
            let mut java_field = Field::new(Java::from(field.ty), field.name.clone());
            java_field.modifiers = vec![Modifier::Public];
            class.fields.push(java_field);
        }

        let mut buf = String::new();
        {
            let mut formatter = Formatter::new(&mut buf);
            let mut extra = java::Extra::default();
            extra.package(self.pkg.as_ref());
            java::Java::write_file(class.into_tokens(), &mut formatter, &mut extra, 0)?;
        }

        Ok(buf)
    }
}

pub(crate) struct CallbackGen<'a> {
    pub desc: &'a TraitDesc,
    pub pkg: String
}

// impl<'a> CallbackGen<'a> {
//     pub(crate) fn gen(&self) -> Result<String> {
//         let mut tokens = toks!("package", " ", self.pkg.clone(), ";");
        
//         let mut interface = Interface::new(self.desc.name.clone());
//         interface.modifiers.push(Modifier::Public);
//         interface.extends = toks!(java::imported("java.io", "Serializable"));

//         for method in self.desc.methods.iter() {
//             let mut m = Method::new(method.name.clone());
//             m.modifiers = vec![Modifier::Abstract];
//             m.returns = Java::from(method.return_type);
//             for arg in method.args.iter() {
//                 let arg_ty = match arg.ty {
//                     AstType::Struct => java::imported(self.pkg, arg.name),
//                     AstType::Vec(base) => {
//                         if base == AstBaseType::Struct {
//                             let sub_ty = std::str::replace(arg.origin_ty, "Vec<");
//                             let sub_ty = std::str::replace(arg.origin_ty, ">");
//                             let  java::imported(self.pkg, sub_ty);
//                         } else {

//                         }
//                     }
//                 }
//             }
//             interface.methods.push();
//         }

//         tokens.push(interface.into_tokens());
//         tokens.to_string().chain_err(|| "struct generate failed.")
//     }
// }

impl From<AstType> for Java<'static> {
    fn from(item: AstType) -> Self {
        match item {
            AstType::Boolean => java::BOOLEAN,
            AstType::Byte => java::BYTE,
            AstType::Int => java::INTEGER,
            AstType::Long => java::LONG,
            AstType::Float => java::FLOAT,
            AstType::Double => java::DOUBLE,
            AstType::String => java::imported("java.lang", "String"),
            AstType::Vec(base) => AstType::from(base).to_array(),
            AstType::Void 
            | AstType::Callback 
            | AstType::Struct => java::VOID
        }
    }
}

impl AstType {
    fn to_array(&self) -> Java<'static> {
        let base_name = Java::from(self.clone());
        let mut base_str = String::new();
        {
            let mut formatter = Formatter::new(&mut base_str);
            let mut extra = java::Extra::default();
            base_name.format(&mut formatter, &mut extra, 0);
        }
        base_str.write_str("[]");
        java::local(base_str) 
    }
}