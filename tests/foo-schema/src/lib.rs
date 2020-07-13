use oo_bindgen::*;

mod callback;
mod class;
mod duration;
mod enums;
mod iterator;
mod strings;
mod structure;

pub fn build_lib() -> Result<Library, BindingError> {
    let mut builder = LibraryBuilder::new("foo", semver::Version::parse("1.2.3").unwrap());
    builder.description("Foo is an interesting lib")?;
    builder.license(
        [
            "foo v1.2.3",
            "Copyright (C) 2020 Automatak LLC",
            "",
            "This is my custom license.",
            "These views are not even my own. They belong to nobody.",
            "  - Frumious Scadateer (@scadateer)",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
    )?;

    callback::define(&mut builder)?;
    class::define(&mut builder)?;
    duration::define(&mut builder)?;
    enums::define(&mut builder)?;
    iterator::define(&mut builder)?;
    strings::define(&mut builder)?;
    structure::define(&mut builder)?;

    Ok(builder.build())
}