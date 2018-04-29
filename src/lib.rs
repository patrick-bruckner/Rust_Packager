#[macro_use]
extern crate reexport_proc_macro;
reexport_proc_macro!(rust_packager_derive);
pub extern crate tempfile;

#[macro_export]
macro_rules! attach_bootloader {
    ($f:ident) => {
        #[derive(Bootloader)]
        struct Main;

        fn main() {
            Main::main($f);
        }
    };
}
