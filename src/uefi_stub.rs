//! UEFI Link support

/// The x86_64 assembly files in OpenSSL set a flag called
/// $win64 and automatically include calls to the RtlVirtualUnwind function if
/// NASM is selected as the assembler scheme.
/// example:
///  (vpaes-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (aesni-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (aes-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (p256-x86_64-asm-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (chacha-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (x86_64-mont5-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (x86_64-mont-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (sha256-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (sha512-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (poly1305-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///  (aesni-gcm-x86_64-nasm.obj) : error LNK2001: unresolved external symbol __imp_RtlVirtualUnwind
///
/// example:
///  lld-link: error: undefined symbol: __imp_RtlVirtualUnwind
///
/// This is a work around for it.
#[no_mangle]
#[export_name="__imp_RtlVirtualUnwind"]
pub extern fn RtlVirtualUnwind()
{
}
