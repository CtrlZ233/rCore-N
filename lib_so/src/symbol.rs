use alloc::vec::Vec;
use xmas_elf::sections::SectionData::SymbolTable64;
use xmas_elf::symbol_table::{Entry, Entry64};
use xmas_elf::ElfFile;
pub type P64 = u64;


fn symbol_table<'a>(elf: &ElfFile<'a>) -> &'a [Entry64] {
    match elf.find_section_by_name(".symtab").unwrap().get_data(&elf).unwrap()
    {
        SymbolTable64(dsym) => dsym,
        _ => panic!("corrupted .symtab"),
    }
}

pub fn vdso_table<'a>(elf: &ElfFile<'a>) -> Vec<(&'a str, usize)> {
    let mut res_vec = Vec::new();
    for sym  in symbol_table(elf){
        let name = sym.get_name(elf);
        if name.unwrap().contains("VDSO") {
            res_vec.push((sym.get_name(&elf).unwrap().trim_start_matches("VDSO_"), sym.value() as usize));
        }
    }
    res_vec
}


pub fn get_symbol_addr<'a>(elf: &ElfFile<'a>, symbol_name: &str) -> usize{
    let mut entry = 0 as usize;
    for sym  in symbol_table(elf){
        let name = sym.get_name(elf);
        // log::debug!("{:?}", name);
        if name.unwrap() == symbol_name{
            entry = sym.value() as usize;
        }
    }
    entry
}