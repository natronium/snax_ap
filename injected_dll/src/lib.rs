use pelite::{
    pe::{Pe, Rva},
    pe64::PeView,
};
use retour::static_detour;
use std::ffi::{CStr, c_char};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::core::PCSTR;

use crate::types::{SendEventInfo, TEvent};

mod types {
    use std::os::raw::c_char;

    #[repr(C)]
    pub struct irr_cstr {
        pub array: *const c_char,
        allocated: u32,
        used: u32,
        _allocator: u64,
    }

    #[repr(C)]
    pub struct SendEventInfo {
        _sendEventInfo_vftable: u64,
        _triggerEventInfo_data: [u8; 0x40],
        pub SendEventName: irr_cstr,
    }

    #[repr(C)]
    pub struct TEvent {}
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    install_hooks();
}

type AddItemFn = extern "C" fn(*const (), *const c_char, u64) -> u64;
type SendEventFn = extern "C" fn(*const SendEventInfo, *const TEvent);

static_detour! {
    static on_add_item: extern "C" fn(*const (), *const c_char, u64) -> u64;
    static send_event: extern "C" fn(*const SendEventInfo, *const TEvent);
}

const ADD_ITEM_TO_INVENTORY_RVA: Rva = 0x2f1500;
const SENDEVENTINFO_SENDEVENT_RVA: Rva = 0x2bbff0;

fn install_hooks() {
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

        let send_event_va = view
            .rva_to_va(SENDEVENTINFO_SENDEVENT_RVA)
            .expect("could not calculate Virtual Address from Relative for sendevent");

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

        send_event
            .initialize(
                std::mem::transmute::<u64, SendEventFn>(send_event_va),
                on_send_event,
            )
            .expect("couldn't retour send_event");

        send_event.enable().unwrap();
    }
}

fn on_send_event(info: *const SendEventInfo, tevent: *const TEvent) {
    let event_name = match unsafe { CStr::from_ptr((*info).SendEventName.array).to_str() } {
        Ok(str) => str,
        Err(e) => &e.to_string(),
    };
    println!("Event! {event_name}");
    let result = send_event.call(info, tevent);
    result
}

// fn additem_detour(_this: *const (), path: *const c_char, arg3: u64) -> u64 {

// }
