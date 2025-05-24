use std::{default::Default, env, fs, path::Path};
use typify::{TypeSpace, TypeSpacePatch, TypeSpaceSettings};

fn main() {
    let out_dir = Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf();

    let schema_path = Path::new("schemas/export-v1.schema.json");
    let schema_json = std::fs::read_to_string(&schema_path).unwrap();
    let schema_jsonval: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
    println!("cargo::rerun-if-changed={}", schema_path.display());

    let mut schema_out_file = out_dir.clone();
    schema_out_file.push("schema.json");
    fs::write(schema_out_file, &schema_json).unwrap();

    let schema = serde_json::from_str::<schemars::schema::RootSchema>(&schema_json).unwrap();
    let mut type_space = TypeSpace::new(
        TypeSpaceSettings::default()
            .with_patch(
                "ProxyBotUnifiedExportFormatDraftV1",
                TypeSpacePatch::default().with_rename("UnifiedExport"),
            ),
    );

    type_space.add_root_schema(schema).unwrap();

    let mut codegen_unformatted_out_file = out_dir.clone();
    codegen_unformatted_out_file.push("codegen_blah.rs");
    eprintln!("unformatted: {}", &codegen_unformatted_out_file.display());
    fs::write(codegen_unformatted_out_file, type_space.to_stream().to_string()).unwrap();

    let codegen_contents =
        prettyplease::unparse(&syn::parse2::<syn::File>(type_space.to_stream()).unwrap());

    let schema_id_str = schema_jsonval.get("$id").unwrap().as_str().unwrap();
    let codegen_schemaid =
        prettyplease::unparse(&syn::parse2(quote::quote! {
            pub const SCHEMA_ID: &'static str = #schema_id_str;
        }).unwrap());

    let mut codegen_out_file = out_dir.clone();
    codegen_out_file.push("codegen.rs");
    fs::write(codegen_out_file, vec![ codegen_schemaid, codegen_contents, ].join("\n")).unwrap();
}
