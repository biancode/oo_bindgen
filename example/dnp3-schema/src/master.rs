use oo_bindgen::callback::InterfaceHandle;
use oo_bindgen::class::ClassDeclarationHandle;
use oo_bindgen::doc::DocBuilder;
use oo_bindgen::native_enum::*;
use oo_bindgen::native_function::*;
use oo_bindgen::*;

pub fn define(
    lib: &mut LibraryBuilder,
    master_class: ClassDeclarationHandle,
    read_handler: InterfaceHandle,
    decode_log_level_enum: NativeEnumHandle,
) -> Result<ClassDeclarationHandle, BindingError> {
    let destroy_fn = lib
        .declare_native_function("master_destroy")?
        .param("master", Type::ClassRef(master_class.clone()), "Master to destroy")?
        .return_type(ReturnType::void())?
        .doc(DocBuilder::new().text("Remove and destroy a master.").warn("This method must NOT be called from within the tokio runtime"))?
        .build()?;

    // Association creation
    let association_class = lib.declare_class("Association")?;

    let event_classes = lib.declare_native_struct("EventClasses")?;
    let event_classes = lib
        .define_native_struct(&event_classes)?
        .add("class1", Type::Bool)?
        .add("class2", Type::Bool)?
        .add("class3", Type::Bool)?
        .build();

    let auto_time_sync_enum = lib
        .define_native_enum("AutoTimeSync")?
        .push("None", "Do not perform automatic timesync")?
        .push("LAN", "Perform automatic timesync with Record Current Time (0x18) function code")?
        .push("NonLAN", "Perform automatic timesync with Delay Measurement (0x17) function code")?
        .doc("Automatic time synchronization configuration")?
        .build()?;

    let association_configuration = lib.declare_native_struct("AssociationConfiguration")?;
    let association_configuration = lib
        .define_native_struct(&association_configuration)?
        .add("disable_unsol_classes", Type::Struct(event_classes.clone()))?
        .add("enable_unsol_classes", Type::Struct(event_classes))?
        .add("auto_time_sync", Type::Enum(auto_time_sync_enum))?
        .build();

    let association_handlers = lib.declare_native_struct("AssociationHandlers")?;
    let association_handlers = lib
        .define_native_struct(&association_handlers)?
        .add("integrity_handler", Type::Interface(read_handler.clone()))?
        .add("unsolicited_handler", Type::Interface(read_handler.clone()))?
        .add("default_poll_handler", Type::Interface(read_handler))?
        .build();

    let add_association_fn = lib
        .declare_native_function("master_add_association")?
        .param("master", Type::ClassRef(master_class.clone()), "Master to add the association to")?
        .param("address", Type::Uint16, "DNP3 data-link address of the remote outstation")?
        .param("config", Type::Struct(association_configuration), "Association configuration")?
        .param("handlers", Type::Struct(association_handlers), "Handlers to call when receiving point data")?
        .return_type(ReturnType::new(Type::ClassRef(association_class.clone()), "Handle to the created association or NULL if an error occured"))?
        .doc("Add an association to the master")?
        .build()?;

    let set_decode_log_level_fn = lib
        .declare_native_function("master_set_decode_log_level")?
        .param("master", Type::ClassRef(master_class.clone()), "Master to modify")?
        .param("level", Type::Enum(decode_log_level_enum), "Decode log level")?
        .return_type(ReturnType::void())?
        .doc("Set the master decoding level for log messages")?
        .build()?;

    lib.define_class(&master_class)?
        .destructor(&destroy_fn)?
        .method("AddAssociation", &add_association_fn)?
        .method("SetDecodeLogLevel", &set_decode_log_level_fn)?
        .build();

    Ok(association_class)
}
