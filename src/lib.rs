use std::path::Path;
use xsd_parser::{Error, Generator, Interpreter, Optimizer, Parser, DataTypes};
use xsd_parser::config::GeneratorFlags;
use xsd_parser::models::data::{ComplexData, ComplexDataStruct, DataTypeVariant};
use xsd_parser::pipeline::parser::resolver::FileResolver;

fn generate_data_struct(data_struct: &ComplexDataStruct, _data_types: &DataTypes) {
   if data_struct.tag_name.is_some() {
       println!("\nStruct name:\t{}", data_struct.tag_name.as_ref().unwrap());
   }
    let _type_ident = &data_struct.type_ident;

    let attributes = &data_struct.attributes;
    for attr in attributes {
        println!("Attribute:\t{}", attr.tag_name);
    }

    let fields = data_struct.elements().iter();
    for field in fields {
        println!("Field name:\t\t{}", field.tag_name);
    }

    let content = data_struct.content();
    if content.is_some() {
        let content = data_struct.content().unwrap();
        if content.is_simple {
            println!("Content is simple");
        }
    }

    println!("done");
}

fn generate_data(data: &ComplexData, data_types: &DataTypes) {
    match data {
        ComplexData::Enum {
            type_: _type_,
            content_type: _content_type
        } => {
            unimplemented!("Enums are not implemented yet");
        },
        ComplexData::Struct {
            type_: data_struct,
            content_type
        } => {
            generate_data_struct(data_struct, &data_types);

            if content_type.is_some() {
                let val = content_type.as_deref().unwrap();
                generate_data(val, &data_types)
            }
        }
    }
}

fn generate_xml_data(data_types: &DataTypes) {

    for data in &data_types.items {
        let data_type = data.1;

        match &data_type.variant {
            DataTypeVariant::BuildIn(_) => (),
            DataTypeVariant::Custom(_) => println!("Custom"),
            DataTypeVariant::Union(_) => println!("Union"),
            DataTypeVariant::Dynamic(_) => println!("Dynamic"),
            DataTypeVariant::Reference(_) => (),
            DataTypeVariant::Enumeration(_) => println!("Enumeration"),
            DataTypeVariant::Complex(x) => generate_data(x, &data_types),
        }
    }
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
