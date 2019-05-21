use errors::*;
use ast::contract::desc::StructDesc;
use ast::contract::desc::TraitDesc;
use ast::types::AstType;
use ast::types::AstBaseType;
use genco::java::{self, *};
use genco::IntoTokens;
use genco::Formatter;
use genco::Custom;
use genco::Tokens;
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

        to_java_file(self.pkg.as_ref(), class.into_tokens())
    }
}

pub(crate) struct CallbackGen<'a> {
    pub desc: &'a TraitDesc,
    pub pkg: String
}

impl<'a> CallbackGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut interface = Interface::new(self.desc.name.clone());
        interface.modifiers.push(Modifier::Public);
        interface.extends = toks!(java::imported("java.io", "Serializable"));

        for method in self.desc.methods.iter() {
            let mut m = Method::new(method.name.clone());
            m.modifiers = vec![];
            m.returns = match method.return_type {
                AstType::Struct => {
                    java::imported(self.pkg.as_ref(), method.origin_return_ty.as_ref())
                }
                _ => {
                    Java::from(method.return_type)
                }
            };
            for arg in method.args.iter() {
                let arg_ty = match arg.ty {
                    AstType::Struct => java::imported(self.pkg.as_ref(), arg.name.as_ref()),
                    AstType::Vec(base) => {
                        match base {
                            AstBaseType::Struct => {
                                let sub_ty = arg.origin_ty.replace("Vec<", "").replace(">", "");
                                java::imported(self.pkg.as_ref(), sub_ty)
                            }
                            _ => Java::from(arg.ty)
                        }
                    }
                    _ => Java::from(arg.ty)
                };
                let mut argument = java::Argument::new(arg_ty, arg.name.as_ref());
                argument.modifiers = vec![];

                m.arguments.push(argument);
            }
            interface.methods.push(m);
        }

        to_java_file(self.pkg.as_ref(), interface.into_tokens())
    }
}

pub(crate) struct TraitGen<'a> {
    pub desc: &'a TraitDesc,
    pub pkg: String,
    pub so_name: String,
    pub ext_libs: String,
    pub callbacks: Vec<TraitDesc>
}

impl<'a> TraitGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut class = Class::new(self.desc.name.clone());
        class.modifiers = vec![Modifier::Public];
        class.implements.push(java::imported("java.io", "Serializable"));

        let mut body = Tokens::new();
        body.push("static {");
        body.nested({
            let mut load_lib_tokens = Tokens::new();
            load_lib_tokens.push(toks!("System.loadLibrary(\"", self.so_name.clone(), "\");"));
            let ext_libs = self.ext_libs.split(",").collect::<Vec<&str>>();
            for ext_lib in ext_libs.iter() {
                load_lib_tokens.push(toks!("System.loadLibrary(\"", ext_lib.clone(), "\");"));
            }
            load_lib_tokens
        });
        body.push("}");
        class.body = body;

        let mut index_field = Field::new(java::imported("java.util.concurrent.atomic", "AtomicLong"), "globalIndex");
        index_field.initializer("new AtomicLong(0)");
        index_field.modifiers = vec![Modifier::Private, Modifier::Static];
        class.fields.push(index_field);

        let callbacks_ty = java::imported("java.util.concurrent", "ConcurrentHashMap")
            .with_arguments(vec![java::LONG, java::imported("java.lang", "Object")]);
        let mut callbacks_field = Field::new(callbacks_ty, "globalCallbacks");
        callbacks_field.initializer("new ConcurrentHashMap<>()");
        callbacks_field.modifiers = vec![Modifier::Private, Modifier::Static];
        class.fields.push(callbacks_field);

        let mut free_method = Method::new("free_callback");
        free_method.modifiers = vec![Modifier::Public, Modifier::Static];
        free_method.arguments = vec![java::Argument::new(java::LONG, "index")];
        free_method.body = toks!("globalCallbacks.remove(index);");
        class.methods.push(free_method);

        let mut sel_callbacks = vec![];

        for method in self.desc.methods.iter() {
            let mut m = java::Method::new(method.name.as_ref());
            m.modifiers = vec![Modifier::Public, Modifier::Static];
            match method.return_type {
                AstType::Void => (),
                AstType::Struct => {
                    m.returns = java::imported(self.pkg.as_ref(), method.origin_return_ty.clone());
                }
                AstType::Vec(_) => {
                    m.returns = Java::from(method.return_type);
                }
                _ => { 
                    m.returns = method.return_type.into();
                }
            }

            for arg in method.args.iter() {
                match arg.ty {
                    AstType::Void => (),
                    AstType::Callback => {
                        let argument = Argument::new(java::imported(self.pkg.as_ref(), arg.origin_ty.as_ref()), arg.name.as_ref());
                        m.arguments.push(argument);
                        let callback = self.callbacks.iter()
                            .filter(|callback| callback.name == arg.origin_ty)
                            .collect::<Vec<&TraitDesc>>();
                        if callback.len() > 0 && sel_callbacks.contains(&callback[0]) {
                            sel_callbacks.push(callback[0]);
                        }
                    }
                    AstType::Vec(base) => {
                        match base {
                            AstBaseType::Byte => {
                                let argument = Argument::new(Java::from(arg.ty), arg.name.as_ref());
                                m.arguments.push(argument);
                            }
                            AstBaseType::Struct => {
                                let argument = Argument::new(arg.ty.to_struct_array(self.pkg.as_ref(), arg.origin_ty.as_ref()), arg.name.as_ref());
                                m.arguments.push(argument); 
                            }
                            _ => {
                                let argument = Argument::new(arg.ty.to_boxed_array(), arg.name.as_ref());
                                m.arguments.push(argument);              
                            }
                        }
                    }
                    _ => {
                        let argument = Argument::new(Java::from(arg.ty), arg.name.as_ref());
                        m.arguments.push(argument);
                    }
                }
            }
            class.methods.push(m);
        }

        to_java_file(self.pkg.as_ref(), class.into_tokens())
    }
}

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
            AstType::Void => java::VOID,
            AstType::Callback 
            | AstType::Struct => panic!("not suppported") //TODO
        }
    }
}

impl AstType {
    fn to_array(&self) -> Java<'static> {
        let base_name = Java::from(self.clone());
        self.to_java_array(base_name, false)
    }

    fn to_boxed_array(&self) -> Java<'static> {
        let base_name = Java::from(self.clone());
        self.to_java_array(base_name, true) 
    }

    fn to_struct_array<'a>(&self, pkg: &'a str, struct_name: &'a str) -> Java<'static> {
        let base_name = imported(pkg.to_owned(), struct_name.to_owned());
        self.to_java_array(base_name, false)
    }

    fn to_java_array(&self, java: Java<'static>, boxed: bool) -> Java<'static> {
        let mut base_str = String::new();
        {
            let mut formatter = Formatter::new(&mut base_str);
            let mut extra = java::Extra::default();
            let level = if boxed {1} else {0};
            let _ = java.format(&mut formatter, &mut extra, level);
        }
        let _ = base_str.write_str("[]");
        java::local(base_str) 
    }
}

fn to_java_file(pkg: &str, tokens: Tokens<Java>) -> Result<String> {
    let mut buf = String::new();
    {
        let mut formatter = Formatter::new(&mut buf);
        let mut extra = java::Extra::default();
        extra.package(pkg.as_ref());
        java::Java::write_file(tokens, &mut formatter, &mut extra, 0)?;
    } 
    Ok(buf)
}