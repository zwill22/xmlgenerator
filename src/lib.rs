use fake::{Fake, Faker};
use std::cmp::PartialEq;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::string::String;
use syn::__private::ToTokens;
use syn::{
    AngleBracketedGenericArguments, Field, File, GenericArgument, Item, ItemStruct,
    ItemType, PathArguments, PathSegment, Type, TypePath,
};
use xml_builder::{XMLBuilder, XMLElement, XMLVersion};
use xsd_parser::config::GeneratorFlags;
use xsd_parser::pipeline::parser::resolver::FileResolver;
use xsd_parser::{
    DataTypes, Error, Generator, Interpreter, Optimizer, Parser, Renderer, TypesRenderStep,
};
pub fn format_code_string(code: String) -> Result<String, Error> {
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

struct FieldType {
    name: String,
    min_occurrences: Option<u64>,
    max_occurrences: Option<u64>,
}

impl PartialEq for FieldType {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.min_occurrences != other.min_occurrences {
            return false;
        }

        if self.max_occurrences != other.max_occurrences {
            return false;
        }

        true
    }
}

fn sort_args(args: &AngleBracketedGenericArguments) -> FieldType {
    let mut output = None;

    for arg in args.args.iter() {
        let result = match arg {
            GenericArgument::Lifetime(_) => unimplemented!("Lifetime argument"),
            GenericArgument::Type(x) => get_field_type(x),
            GenericArgument::Const(_) => unimplemented!("Constant argument"),
            GenericArgument::AssocType(_) => unimplemented!("Associative argument"),
            GenericArgument::AssocConst(_) => unimplemented!("Associative argument"),
            GenericArgument::Constraint(_) => unimplemented!("Constraint argument"),
            _ => unimplemented!("Unknown argument"),
        };

        if result.is_some() {
            if output.is_some() {
                unimplemented!("Multiple arguments are not supported yet");
            }

            output = result;
        }
    }

    if output.is_none() {
        panic!("No arguments found");
    }

    output.unwrap()
}

fn get_arguments(segment: &PathSegment) -> FieldType {
    match &segment.arguments {
        PathArguments::None => unimplemented!("No path arguments"),
        PathArguments::AngleBracketed(x) => sort_args(x),
        PathArguments::Parenthesized(_) => unimplemented!("Parenthesized path arguments"),
    }
}

fn generate_field_type(type_path: &TypePath) -> FieldType {
    let stream = &type_path.path.segments;
    for segment in stream.iter() {
        let seg_type = segment.ident.to_string();
        let mut field_type = get_arguments(segment);

        if seg_type == "Option" {
            field_type.min_occurrences = Some(0);
            field_type.max_occurrences = Some(1);
        } else if seg_type == "Vec" {
            field_type.min_occurrences = Some(0);
            field_type.max_occurrences = None;
        } else {
            unimplemented!("Unknown type: {}", seg_type);
        }

        return field_type;
    }

    panic!("No type found");
}

fn find_field_type(type_path: &TypePath) -> FieldType {
    let mut name = None;

    let ident = &type_path.path.get_ident();
    if ident.is_some() {
        name = Some(ident.unwrap().to_string());
    }

    let qself = type_path.qself.clone();
    if qself.is_some() {
        name = Some(qself.unwrap().ty.deref().into_token_stream().to_string());
    }

    if name.is_some() {
        return FieldType {
            name: name.unwrap(),
            min_occurrences: None,
            max_occurrences: None,
        };
    }

    generate_field_type(type_path)
}

fn get_field_type(field_type: &Type) -> Option<FieldType> {
    match field_type {
        Type::Array(_) => unimplemented!("Field type: Array"),
        Type::BareFn(_) => unimplemented!("Field type: BareFn"),
        Type::Group(_) => unimplemented!("Field type: Group"),
        Type::ImplTrait(_) => unimplemented!("Field type: ImplTrait"),
        Type::Infer(_) => unimplemented!("Field type: Infer"),
        Type::Macro(_) => unimplemented!("Field type: Macro"),
        Type::Never(_) => unimplemented!("Field type: Never"),
        Type::Paren(_) => unimplemented!("Field type: Paren"),
        Type::Path(x) => Option::from(find_field_type(x)),
        Type::Ptr(_) => unimplemented!("Field type: Ptr"),
        Type::Reference(_) => unimplemented!("Field type: Reference"),
        Type::Slice(_) => unimplemented!("Field type: Slice"),
        Type::TraitObject(_) => unimplemented!("Field type: TraitObject"),
        Type::Tuple(_) => unimplemented!("Field type: Tuple"),
        Type::Verbatim(_) => unimplemented!("Field type: Verbatim"),
        _ => unimplemented!("Field type: Other"),
    }
}

struct TypeAlias<'a> {
    name: String,
    value: &'a Type,
    attrs: Vec<String>,
}

fn type_alias(item_type: &ItemType) -> TypeAlias {
    println!(
        "Alias name: {}",
        item_type.ident.to_token_stream().to_string()
    );
    let name = item_type.ident.to_string();
    let value = item_type.ty.deref();
    println!("Alias type: {}", value.to_token_stream().to_string());

    let mut attrs = vec![];
    for attr in item_type.attrs.iter() {
        attrs.push(attr.to_token_stream().to_string());
    }

    TypeAlias { name, value, attrs }
}

fn render(data_types: &DataTypes) -> File {
    let renderer = Renderer::new(data_types).with_step(TypesRenderStep);

    let module = renderer.finish();

    let code = module.code.to_string();

    let output = format_code_string(code).unwrap().to_string();

    syn::parse_file(&*output).unwrap()
}

fn get_type_alias(item: &Item) -> Option<TypeAlias> {
    match item {
        Item::Const(_) => unimplemented!("Item::Const"),
        Item::Enum(_) => unimplemented!("Item::Enum"),
        Item::ExternCrate(_) => unimplemented!("Item::ExternCrate"),
        Item::Fn(_) => unimplemented!("Item::Fn"),
        Item::ForeignMod(_) => unimplemented!("Item::ForeignMod"),
        Item::Impl(_) => unimplemented!("Item::Impl"),
        Item::Macro(_) => unimplemented!("Item::Macro"),
        Item::Mod(_) => unimplemented!("Item::Mod"),
        Item::Static(_) => unimplemented!("Item::Static"),
        Item::Struct(_) => None,
        Item::Trait(_) => unimplemented!("Item::Trait"),
        Item::TraitAlias(_) => unimplemented!("Item::TraitAlias"),
        Item::Type(x) => Option::from(type_alias(x)),
        Item::Union(_) => unimplemented!("Item::Union"),
        Item::Use(_) => unimplemented!("Item::Use"),
        Item::Verbatim(_) => unimplemented!("Item::Verbatim"),
        &_ => unimplemented!("Item::Other"),
    }
}

struct FieldInfo {
    name: String,
    field_type: FieldType,
    attributes: Vec<String>,
}

struct StructInfo {
    name: String,
    attrs: Vec<String>,
    fields: Vec<FieldInfo>,
}

impl PartialEq for FieldInfo {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.field_type != other.field_type {
            return false;
        }

        if self.attributes.len() != other.attributes.len() {
            return false;
        }

        for i in 0..self.attributes.len() {
            if self.attributes[i] != other.attributes[i] {
                return false;
            }
        }

        true
    }
}

impl PartialEq for StructInfo {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }

        if self.attrs.len() != other.attrs.len() {
            return false;
        }
        for i in 0..self.attrs.len() {
            if self.attrs[i] != other.attrs[i] {
                return false;
            }
        }

        if self.fields.len() != other.fields.len() {
            return false;
        }

        for i in 0..self.fields.len() {
            if self.fields[i] != other.fields[i] {
                return false;
            }
        }

        true
    }
}

fn get_field(field: &Field) -> FieldInfo {
    if field.ident.is_none() {
        panic!("Unnamed fields are not supported!");
    }
    let ident = field.ident.as_ref().unwrap();
    let field_name = ident.to_string();
    println!("Field name: {}", field_name);
    let field_type = get_field_type(&field.ty);

    let mut attrs = vec![];
    for attr in field.attrs.iter() {
        attrs.push(attr.into_token_stream().to_string());
    }

    FieldInfo {
        name: field_name,
        field_type: field_type.unwrap(),
        attributes: attrs,
    }
}

fn get_struct_info(struct_item: &ItemStruct) -> StructInfo {
    let struct_token = struct_item.struct_token;
    println!("struct: {}", struct_token.to_token_stream().to_string());

    println!(
        "Visibility: {}",
        struct_item.vis.to_token_stream().to_string()
    );

    let name = struct_item.ident.to_token_stream().to_string();
    println!("Struct name: {}", name);
    let mut attrs = vec![];
    for attr in &struct_item.attrs {
        println!("Attr: {}", attr.to_token_stream().to_string());
        attrs.push(attr.to_token_stream().to_string());
    }

    let field_data = struct_item.fields.iter();
    let mut fields = vec![];
    for field in field_data {
        let field_info = get_field(field);
        fields.push(field_info);
    }

    StructInfo {
        name,
        attrs,
        fields,
    }
}

fn get_struct(item: &Item) -> Option<StructInfo> {
    match item {
        Item::Const(_) => unimplemented!("Item::Const"),
        Item::Enum(_) => unimplemented!("Item::Enum"),
        Item::ExternCrate(_) => unimplemented!("Item::ExternCrate"),
        Item::Fn(_) => unimplemented!("Item::Fn"),
        Item::ForeignMod(_) => unimplemented!("Item::ForeignMod"),
        Item::Impl(_) => unimplemented!("Item::Impl"),
        Item::Macro(_) => unimplemented!("Item::Macro"),
        Item::Mod(_) => unimplemented!("Item::Mod"),
        Item::Static(_) => unimplemented!("Item::Static"),
        Item::Struct(x) => Option::from(get_struct_info(x)),
        Item::Trait(_) => unimplemented!("Item::Trait"),
        Item::TraitAlias(_) => unimplemented!("Item::TraitAlias"),
        Item::Type(_) => None,
        Item::Union(_) => unimplemented!("Item::Union"),
        Item::Use(_) => unimplemented!("Item::Use"),
        Item::Verbatim(_) => unimplemented!("Item::Verbatim"),
        &_ => unimplemented!("Item::Other"),
    }
}

fn get_data(data: &File) -> (Vec<TypeAlias>, Vec<StructInfo>) {
    let mut type_aliases = vec![];
    let mut structs = vec![];
    for item in &data.items {
        let type_result = get_type_alias(&item);
        if type_result.is_some() {
            type_aliases.push(type_result.unwrap());
        }

        let struct_result = get_struct(&item);
        if struct_result.is_some() {
            structs.push(struct_result.unwrap());
        }
    }

    (type_aliases, structs)
}

fn get_field_struct<'a>(structs: &'a Vec<StructInfo>, field: &String) -> Option<&'a StructInfo> {
    for structure in structs.iter() {
        if structure.name == field.deref() {
            return Option::from(structure);
        }
    }

    None
}

fn find_root<'a>(structs: &'a Vec<StructInfo>) -> &'a StructInfo {
    let mut all_fields: Vec<&String> = vec![];
    for structure in structs.iter() {
        for field in structure.fields.iter() {
            if !all_fields.contains(&&field.field_type.name) {
                all_fields.push(&field.field_type.name);
            }
        }
    }
    let mut dep_structs = vec![];
    for field in all_fields.iter() {
        let structure = get_field_struct(&structs, field);
        if structure.is_some() {
            dep_structs.push(structure.unwrap());
        }
    }

    let mut independent_structs = vec![];

    for structure in structs.iter() {
        if !dep_structs.contains(&structure) {
            independent_structs.push(structure);
        }
    }

    if independent_structs.is_empty() {
        panic!("No root structs found!");
    }

    if independent_structs.len() > 1 {
        println!("Multiple independent structs found!");
    }

    for structure in structs.iter() {
        if independent_structs.contains(&structure) {
            return structure;
        }
    }

    panic!("No root structs found!");
}

fn make_fake<Output: fake::Dummy<Faker> + ToString>() -> Option<String> {
    Option::from(Faker.fake::<Output>().to_string())
}

fn get_string(type_name: &String) -> Option<String> {
    match type_name.as_str() {
        "i8" => make_fake::<i8>(),
        "u8" => make_fake::<u8>(),
        "i16" => make_fake::<i16>(),
        "u16" => make_fake::<u16>(),
        "i32" => make_fake::<i32>(),
        "u32" => make_fake::<u32>(),
        "i64" => make_fake::<i64>(),
        "u64" => make_fake::<u64>(),
        "i128" => make_fake::<i128>(),
        "u128" => make_fake::<u128>(),
        "isize" => make_fake::<isize>(),
        "usize" => make_fake::<usize>(),
        "f32" => make_fake::<f32>(),
        "f64" => make_fake::<f64>(),
        "bool" => make_fake::<bool>(),
        "char" => make_fake::<char>(),
        "String" => make_fake::<String>(),
        _ => None,
    }
}

fn get_element(
    field: &FieldInfo,
    structs: &Vec<StructInfo>,
    types: &Vec<TypeAlias>,
) -> Option<XMLElement> {
    for structure in structs {
        if structure.name == field.field_type.name {
            let element = generate_element(structure, structs, types);
            return Option::from(element);
        }
    }

    None
}

fn get_child(
    field: &FieldInfo,
    structs: &Vec<StructInfo>,
    types: &Vec<TypeAlias>,
) -> Option<XMLElement> {
    let value = get_string(&field.field_type.name);
    if value.is_some() {
        let mut child = XMLElement::new(&field.name);
        let _ = child.add_text(value.unwrap());
        return Option::from(child);
    }

    get_element(&field, structs, types)
}

fn generate_element(
    root: &StructInfo,
    structs: &Vec<StructInfo>,
    types: &Vec<TypeAlias>,
) -> XMLElement {
    let name = root.name.clone();
    let mut element = XMLElement::new(&*name);

    for field in root.fields.iter() {
        let child = get_child(&field, &structs, &types);
        if child.is_some() {
            let _ = element.add_child(child.unwrap());
        }
    }

    element
}

fn generate_xml_data(data_types: &DataTypes) {
    let data = render(data_types);

    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let (type_aliases, structs) = get_data(&data);

    let root = find_root(&structs);
    let root_element = generate_element(&root, &structs, &type_aliases);

    let mut writer: Vec<u8> = Vec::new();
    xml.set_root_element(root_element);
    xml.generate(&mut writer).unwrap();

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
