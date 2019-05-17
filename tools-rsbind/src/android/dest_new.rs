use errors::*;
use ast::contract::desc::StructDesc;
use ast::types::AstType;
use genco::java::{self, *};
use genco::IntoTokens;

pub(crate) struct StructGen<'a> {
    pub desc: &'a StructDesc,
    pub pkg: String
}

impl<'a> StructGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut class = Class::new(self.desc.name.clone());
        class.modifiers.push(Modifier::Public);
        class.extends = Some(java::imported("java.io", "Serializable"));
        
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

        class.into_tokens().to_string().chain_err(|| "struct generate failed.")
    }
}