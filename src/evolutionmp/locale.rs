use crate::pattern::MemoryRegion;
use crate::hash::Hash;

type GetText = extern "C" fn(text: *mut (), hash: Hash) -> *const u8;
static mut NATIVE_GET_TEXT: *const () = std::ptr::null();

pub unsafe extern "C" fn get_text(text: *mut (), hash: Hash) -> *const u8 {
    if hash == Hash(0xABB00DEB) {
        return b"Evolution MP\0" as _;
    }
    //crate::info!("getting text for hash 0x{:08X}", hash.0);
    let origin: GetText = std::mem::transmute(NATIVE_GET_TEXT);
    let result = origin(text, hash);
    //crate::info!("got text {} for hash 0x{:08X}", CStr::from_ptr(result as _).to_string_lossy(), hash.0);
    result
}

pub unsafe fn init(mem: &MemoryRegion) {
    let d = mem.find("48 8B CB 8B D0 E8 ? ? ? ? 48 85 C0 0F 95 C0")
        .next().expect("get_text").add(5).detour(get_text as _);
    NATIVE_GET_TEXT = d;

    /*let d2 = mem.find("48 85 C0 75 34 8B 0D")
        .next().expect("get_text_2").offset(-5).set_call(get_text as _);*/
}