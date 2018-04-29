#![recursion_limit = "128"]
extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate serde_derive;
extern crate syn;
extern crate toml;

use proc_macro::TokenStream;
use quote::Tokens;
use syn::Data;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use toml::value;

#[derive(Deserialize)]
struct Cargo {
    package: Package
}
#[derive(Deserialize)]
struct Package {
    metadata: Metadata
}
#[derive(Deserialize)]
struct Metadata {
    rust_packager: Packager
}
#[derive(Deserialize)]
struct Packager {
    files: value::Array
}

fn inject_bootloader(ast: &syn::DeriveInput) -> Tokens {
    match ast.data {
        Data::Enum(_) => panic!(),
        Data::Union(_) => panic!(),
        Data::Struct(_) => {},
    };

    let mut manifest_path = PathBuf::new();
    manifest_path.push(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_path.push("Cargo.toml");

    let mut cargo_file = File::open(manifest_path).unwrap();
    let mut cargo_str = String::new();
    cargo_file.read_to_string(&mut cargo_str).unwrap();
    let cargo: Cargo = toml::from_str(&cargo_str).unwrap();

    let mut values = Vec::<Tokens>::new();

    for entry in cargo.package.metadata.rust_packager.files {
        match entry {
            toml::Value::String(s) => {
                let value = quote!{
                    (#s, include_bytes!(#s)),
                };
                values.push(value);
            },
            _ => {}
        }
    }

    let ident = &ast.ident;

    quote! {
        impl #ident {
            fn setup() {
                use rust_packager::tempfile;

                use std::env;
                use std::fs::File;
                use std::io::Write;
                use std::process::Command;

                let files: Vec<(&str, &[u8])> = vec![#(#values)*];

                let dir = tempfile::tempdir().unwrap();
                for (name, data) in files {
                    let file_path = dir.path().join(&name);
                    {
                        let mut file = File::create(file_path).unwrap();
                        file.write_all(data).unwrap();
                    }
                }

                let exe_path = match env::current_exe() {
                    Ok(exe_path) => exe_path,
                    Err(e) => panic!("{}", e),
                };

                let mut child = Command::new(exe_path)
                                        .current_dir(dir.path())
                                        .env("START_RS", "")
                                        .spawn().unwrap();
                child.wait().unwrap();
            }

            pub fn main(f: fn()) {
                match std::env::var("START_RS") {
                    Ok(_) => f(),
                    Err(_) => #ident::setup(),
                }
            }
        }
    }
}


#[proc_macro_derive(Bootloader, attributes(path))]
pub fn derive_bootloader(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    let gen = inject_bootloader(&ast);
    gen.into()
}
