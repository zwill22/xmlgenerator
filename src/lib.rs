use std::path::Path;
use xsd_parser::config::GeneratorFlags;
use xsd_parser::pipeline::parser::resolver::FileResolver;
use xsd_parser::{Error, Generator, Interpreter, Optimizer, Parser, DataTypes, Renderer, TypesRenderStep};
use std::io::Write;
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use std::process::{Command, Output, Stdio};
use syn::{Field, File, Item, ItemStruct, Type};
use syn::__private::ToTokens;

pub fn rustfmt_pretty_print(code: String) -> Result<String, Error> {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();

    write!(stdin, "{code}")?;
    stdin.flush()?;
    drop(stdin);

    let Output {
        status,
        stdout,
        stderr,
    } = child.wait_with_output()?;

    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);

    if !status.success() {
        let code = status.code();
        match code {
            Some(code) => {
                if code != 0 {
                    panic!("The `rustfmt` command failed with return code {code}!\n{stderr}");
                }
            }
            None => {
                panic!("The `rustfmt` command failed!\n{stderr}")
            }
        }
    }

    Ok(stdout.into())
}

fn generate_value(field_type: &Type) -> String {
    match field_type {
        Type::Array(_) => {}
        Type::BareFn(_) => {}
        Type::Group(_) => {}
        Type::ImplTrait(_) => {}
        Type::Infer(_) => {}
        Type::Macro(_) => {}
        Type::Never(_) => {}
        Type::Paren(_) => {}
        Type::Path(_) => {}
        Type::Ptr(_) => {}
        Type::Reference(_) => {}
        Type::Slice(_) => {}
        Type::TraitObject(_) => {}
        Type::Tuple(_) => {}
        Type::Verbatim(_) => {}
        _ => {}
    }

    "Value".to_string()
}

fn sort_field(field: &Field) -> XMLElement {
    if field.ident.is_none() {
        panic!("Unnamed fields are not supported!");
    }
    let ident = field.ident.as_ref().unwrap();
    let string = ident.to_string();
    let field_type = &field.ty;

    let value = generate_value(field_type);

    let mut element = XMLElement::new(&*string);
    element.add_text(value).unwrap();

    element
}

fn sort_struct(struct_type: &ItemStruct) -> XMLElement {
    let mut element = XMLElement::new(&*struct_type.ident.to_string());
    for attr in &struct_type.attrs {
        println!("Struct attribute: {}", attr.into_token_stream());
    }

    let fields = struct_type.fields.iter();
    for field in fields {
        let field_element = sort_field(field);
        let _ = element.add_child(field_element);
    }

    element
}

fn sort_item(item: &Item) -> Option<XMLElement> {
    let result: Option<XMLElement> = match item {
        Item::Const(_) => unimplemented!("Item::Const"),
        Item::Enum(_) => unimplemented!("Item::Enum"),
        Item::ExternCrate(_) => unimplemented!("Item::ExternCrate"),
        Item::Fn(_) => unimplemented!("Item::Fn"),
        Item::ForeignMod(_) => unimplemented!("Item::ForeignMod"),
        Item::Impl(_) => unimplemented!("Item::Impl"),
        Item::Macro(_) => unimplemented!("Item::Macro"),
        Item::Mod(_) => unimplemented!("Item::Mod"),
        Item::Static(_) => unimplemented!("Item::Static"),
        Item::Struct(x) => Option::from(sort_struct(x)),
        Item::Trait(_) => unimplemented!("Item::Trait"),
        Item::TraitAlias(_) => unimplemented!("Item::TraitAlias"),
        Item::Type(_) => None,
        Item::Union(_) => unimplemented!("Item::Union"),
        Item::Use(_) => unimplemented!("Item::Use"),
        Item::Verbatim(_) => unimplemented!("Item::Verbatim"),
        &_ => todo!()
    };

    result
}

fn render(data_types: &DataTypes) -> File {
    let renderer = Renderer::new(data_types)
        .with_step(TypesRenderStep);

    let module = renderer.finish();

    let code = module.code.to_string();

    let output = rustfmt_pretty_print(code).unwrap().to_string();

    syn::parse_file(&*output).unwrap()
}

fn generate_xml_data(data_types: &DataTypes) {
    let data = render(data_types);

    for attr in data.attrs {
        println!("Attr: {}", attr.into_token_stream());
    }

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut elements: Vec<XMLElement> = Vec::new();
    for item in data.items {
        let result = sort_item(&item);
        if result.is_some() {
            elements.push(result.unwrap());
        }
    }

    let mut writer: Vec<u8> = Vec::new();
    if elements.len() == 1 {
        xml.set_root_element(elements.pop().unwrap());
        xml.generate(&mut writer).unwrap();
    } else {
        for element in elements {
            element.render(&mut writer, false,true, true, true).unwrap()
        }
    }

    println!("{}", String::from_utf8(writer).unwrap());
}

pub fn generate_xml(filepath: Box<Path>) -> Result<String, Error> {
    let schemas = Parser::new()
        .with_resolver(FileResolver::new())
        .with_default_namespaces()
        .add_schema_from_file(filepath.canonicalize()?)?
        .finish();

    let meta_types = Interpreter::new(&schemas)
            .with_buildin_types()?
            .with_default_typedefs()?
            .with_xs_any_type()?
            .finish()?;

    let optimised_metatypes = Optimizer::new(meta_types)
            .remove_empty_enum_variants()
            .remove_empty_enums()
            .remove_duplicate_union_variants()
            .remove_empty_unions()
            .use_unrestricted_base_type()
            .convert_dynamic_to_choice()
            .flatten_complex_types()
            .flatten_unions()
            .merge_enum_unions()
            .resolve_typedefs()
            .remove_duplicates()
            .merge_choice_cardinalities()
            .finish();

    let data_types = Generator::new(&optimised_metatypes)
            .flags(GeneratorFlags::all())
            .generate_named_types()?
            .finish();


    generate_xml_data(&data_types);


    Ok("".to_string())
}
