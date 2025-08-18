use pelite::{
    pe::{Pe, Rva},
    pe64::PeView,
};
use retour::static_detour;
use std::ffi::{CStr, c_char};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::core::PCSTR;

type AddItemFn = extern "C" fn(*const (), *const c_char, u64) -> u64;

static_detour! {
    static on_add_item: extern "C" fn(*const (), *const c_char, u64) -> u64;
}

const ADD_ITEM_TO_INVENTORY_RVA: Rva = 0x2f1500;

#[unsafe(no_mangle)]
extern "C" fn install_hooks() {
    // Bugsnax.exe add_item_to_inventory lives at 0x7ff79af91500
    //  or .text (0x7ff79aca1000) + 0x2f0500
    //  or base  (0x7ff79aca0000) + 0x2f1500

    unsafe {
        let snax_exe_handle =
            GetModuleHandleA(PCSTR(std::ptr::null())).expect("could not find Bugsnax.exe module");
        let view = PeView::module(snax_exe_handle.0 as *const u8);

        println!("snax exe handle: {:?}", snax_exe_handle.0);
        let add_item_va = view
            .rva_to_va(ADD_ITEM_TO_INVENTORY_RVA)
            .expect("could not calculate Virtual Address from Relative for add_item_to_inventory");

        on_add_item
            .initialize(
                std::mem::transmute::<u64, AddItemFn>(add_item_va),
                |a, b, c| {
                    let name = CStr::from_ptr(b)
                        .to_str()
                        .expect("could not convert item def path into str");
                    print!("add_item! arg1: {a:?} arg2: {name} arg3: {c} ");
                    let result = on_add_item.call(a, b, c);
                    println!("returned: {result}");
                    result
                },
            )
            .expect("could not retour add_item_to_inventory");

        on_add_item
            .enable()
            .expect("could not enable on_add_item detour");
    }
}

// fn additem_detour(_this: *const (), path: *const c_char, arg3: u64) -> u64 {

// }
