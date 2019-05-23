use ast::contract::desc::StructDesc;
use ast::contract::desc::TraitDesc;
use ast::types::AstBaseType;
use ast::types::AstType;
use errors::*;
use genco::java::{self, *};
use genco::Custom;
use genco::Formatter;
use genco::IntoTokens;
use genco::Tokens;
use std::fmt::Write;

pub(crate) struct StructGen<'a> {
    pub desc: &'a StructDesc,
    pub pkg: String,
}

impl<'a> StructGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut class = Class::new(self.desc.name.clone());
        class.modifiers.push(Modifier::Public);
        class
            .implements
            .push(java::imported("java.io", "Serializable"));

        for field in self.desc.fields.iter() {
            let field_ty = JavaType::new(field.ty, self.pkg.clone(), field.origin_ty.clone());
            let mut java_field = Field::new(Java::from(field_ty), field.name.clone());
            java_field.modifiers = vec![Modifier::Public];
            class.fields.push(java_field);
        }

        to_java_file(self.pkg.as_ref(), class.into_tokens())
    }
}

pub(crate) struct CallbackGen<'a> {
    pub desc: &'a TraitDesc,
    pub pkg: String,
}

impl<'a> CallbackGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut interface = Interface::new(self.desc.name.clone());
        interface.modifiers.push(Modifier::Public);
        interface.extends = toks!(java::imported("java.io", "Serializable"));

        for method in self.desc.methods.iter() {
            let mut m = Method::new(method.name.clone());
            m.modifiers = vec![];
            m.returns = Java::from(JavaType::new(
                method.return_type,
                self.pkg.clone(),
                method.origin_return_ty.clone(),
            ));
            for arg in method.args.iter() {
                let arg_ty = Java::from(JavaType::new(
                    arg.ty,
                    self.pkg.clone(),
                    arg.origin_ty.clone(),
                ));
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
    pub callbacks: Vec<TraitDesc>,
}

impl<'a> TraitGen<'a> {
    pub(crate) fn gen(&self) -> Result<String> {
        let mut class = Class::new(self.desc.name.clone());
        class.modifiers = vec![Modifier::Public];
        class
            .implements
            .push(java::imported("java.io", "Serializable"));

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

        let mut index_field = Field::new(
            java::imported("java.util.concurrent.atomic", "AtomicLong"),
            "globalIndex",
        );
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

        let methods = self.desc.methods.clone();
        let methods = methods.into_iter();
        for method in methods {
            let mut m = java::Method::new(method.name.clone());
            m.modifiers = vec![Modifier::Public, Modifier::Static];
            
            let return_ty = JavaType::new(
                method.return_type.clone(),
                self.pkg.clone(),
                method.origin_return_ty.clone(),
            );
            m.returns = Java::from(return_ty.clone());

            let mut method_body: Tokens<Java> = Tokens::new();
            for arg in method.args.clone().into_iter() {
                // Add arguments
                match arg.ty {
                    AstType::Void => (),
                    _ => {
                        let java =
                            JavaType::new(arg.ty.clone(), self.pkg.clone(), arg.origin_ty.clone());
                        let mut argument = Argument::new(java, arg.name.clone());
                        argument.modifiers = vec![];
                        m.arguments.push(argument);
                    }
                }

                // Select the callbacks in arguments
                match arg.ty {
                    AstType::Callback => {
                        let callback = self
                            .callbacks
                            .iter()
                            .filter(|callback| callback.name == arg.origin_ty)
                            .collect::<Vec<&TraitDesc>>();
                        if callback.len() > 0 && sel_callbacks.contains(&callback[0]) {
                            sel_callbacks.push(callback[0]);
                        }
                    }
                    _ => (),
                }
            }
            
            // Argument convert
            for arg in method.args.clone().into_iter() {
                let converted = format!("r_{}", &arg.name);
                match arg.ty {
                    AstType::Void => (),
                    AstType::Callback => {
                        let index_name = format!("{}_callback_index", &arg.name);
                        method_body.push(toks!("long ", index_name.clone(), " = globalIndex.incrementAndGet()", ";"));
                        method_body.push(toks!("globalCallbacks.put(", index_name.clone(), ",", arg.name.clone(), ");"));
                        method_body.push(toks!("long ", converted.clone(), " = ", index_name.clone(), ";"));
                    }
                    AstType::Boolean => {
                        method_body.push(toks!("int ", converted.clone(), " = ", arg.name.clone(), " ? 1 : 0;"));
                    }
                    AstType::Vec(base) => {
                        match base {
                            AstBaseType::Byte => {
                                let java = JavaType::new(arg.ty.clone(), self.pkg.clone(), arg.origin_ty.clone());
                                let java = Java::from(java);
                                method_body.push(toks!(java, " ", converted.clone(),  " = ", arg.name.clone(), ";"));
                            }
                            _ => {
                                let json_cls = java::imported("com.alibaba.fastjson", "JSON");
                                method_body.push(toks!("String ", converted.clone(), " = ", json_cls, ".toJSONString(", arg.name.clone(), ");"));
                            }
                        }
                    }
                    _ => {
                        let java = JavaType::new(arg.ty.clone(), self.pkg.clone(), arg.origin_ty.clone());
                        let java = Java::from(java);
                        method_body.push(toks!(java, " ", converted, " = ", arg.name.clone(), ";"));
                    }
                }
            }

            // Call native method
            let return_java_ty = m.returns.clone(); 
            match return_ty.ast_type.clone() {
                AstType::Void => {
                    method_body.push(toks!("native_", method.name.clone(), "("));
                }
                _ => {
                    method_body.push(toks!(return_java_ty, " ret = native_", method.name.clone(), "("));
                }
            }

            for (index, item) in method.args.clone().into_iter().enumerate() {
                let converted = format!("r_{}", &item.name);
                if index == method.args.len() - 1 {
                    method_body.append(toks!(converted));
                } else {
                    method_body.append(toks!(converted, ", "));
                }
            }
            method_body.append(toks!(");"));

            // Return type convert
            match return_ty.ast_type.clone() {
                AstType::Void => (),
                AstType::Vec(base) => {
                    match base {
                        AstBaseType::Byte => {
                            method_body.push(toks!("return ret;"));
                        }
                        _ => {
                            let sub_ty = return_ty.clone().get_base_ty();
                            let json = java::imported("com.alibaba.fastjson", "JSON");
                            let list_type = java::imported("java.util", "List").with_arguments(vec![sub_ty]);
                            method_body.push(toks!(list_type, " list = ", json, ".parseArray(ret, ", sub_ty.clone(), ".class;"));
                            method_body.push(toks!(return_ty.clone().to_array(), " array = new ", sub_ty, "[list.size()];"));
                            mothod_body.push(toks!("return list.toArray(array);"));
                        }
                    }
                }
            }

            m.body = method_body;

            class.methods.push(m);
        }

        to_java_file(self.pkg.as_ref(), class.into_tokens())
    }
}

#[derive(Clone)]
struct JavaType {
    pub ast_type: AstType,
    pub pkg: String,
    pub origin_ty: String,
}

impl JavaType {
    pub(crate) fn new(ast_type: AstType, pkg: String, origin_ty: String) -> JavaType {
        JavaType {
            ast_type,
            pkg,
            origin_ty,
        }
    }

    pub(crate) fn to_array(&self) -> Java<'static> {
        let base_name = Java::from(self.clone());
        self.to_java_array(base_name, false)
    }

    pub(crate) fn to_boxed_array(&self) -> Java<'static> {
        let base_name = Java::from(self.clone());
        self.to_java_array(base_name, true)
    }

    pub(crate) fn to_transfer(&self) -> Java<'static> {
        match self.ast_type {
            AstType::Boolean => java::INTEGER,
            AstType::Vec(base) => {
                match base {
                    AstBaseType::Byte => {
                        Java::from(self.clone())
                    }
                    _ => {
                        Java::from(JavaType::new(self.ast_type, self.pkg.clone(), self.origin_ty.clone()))
                    }
                }
            }
            _ => Java::from(self.clone()),
        }
    }
    
    /// If JavaType is an Vec(base), we will return base, else we will return itself.
    pub(crate) fn get_base_ty(&self) -> Java<'static> {
        match self.ast_type {
            AstType::Vec(base) => {
                match base {
                    AstBaseType::Struct => {
                        let sub_origin_ty = self.origin_ty.replace("Vec<", "").replace(">", "");
                        java::imported(self.pkg.clone(), sub_origin_ty)
                    }
                    _ => Java::from(JavaType::new(AstType::from(base), self.pkg.clone(), self.origin_ty.clone()))
                }
            }
            _ => {
                Java::from(self.clone())
            }
        }
    }

    fn to_java_array(&self, java: Java<'static>, boxed: bool) -> Java<'static> {
        let mut base_str = String::new();
        {
            let mut formatter = Formatter::new(&mut base_str);
            let mut extra = java::Extra::default();
            let level = if boxed { 1 } else { 0 };
            let _ = java.format(&mut formatter, &mut extra, level);
        }
        let _ = base_str.write_str("[]");
        java::local(base_str)
    }
}

impl From<JavaType> for Java<'static> {
    fn from(item: JavaType) -> Self {
        match item.ast_type {
            AstType::Boolean => java::BOOLEAN,
            AstType::Byte => java::BYTE,
            AstType::Int => java::INTEGER,
            AstType::Long => java::LONG,
            AstType::Float => java::FLOAT,
            AstType::Double => java::DOUBLE,
            AstType::String => java::imported("java.lang", "String"),
            AstType::Vec(base) => match base {
                AstBaseType::Struct => {
                    let sub_origin_ty = item.origin_ty.replace("Vec<", "").replace(">", "");
                    JavaType::new(AstType::from(base), item.pkg.clone(), sub_origin_ty.clone())
                        .to_array()
                }
                // Byte array is not transferred by json, so we don't use boxed array.
                AstBaseType::Byte => JavaType::new(
                    AstType::from(base),
                    item.pkg.clone(),
                    item.origin_ty.clone(),
                )
                .to_array(),
                // Why we use boxed array, because we use json to transfer array, 
                // and it is translated to list, and then we need to change it to array(boxed).
                _ => JavaType::new(
                    AstType::from(base),
                    item.pkg.clone(),
                    item.origin_ty.clone(),
                )
                .to_boxed_array(),
            },
            AstType::Void => java::VOID,
            AstType::Callback | AstType::Struct => {
                java::imported(item.pkg.clone(), item.origin_ty.clone())
            }
        }
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
