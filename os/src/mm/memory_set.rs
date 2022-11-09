use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{MEMORY_END, PAGE_SIZE, TRACE_SIZE, TRAMPOLINE, TRAP_CONTEXT, UNFI_SCHE_BUFFER, USER_STACK_SIZE};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::*;
use riscv::asm::sfence_vma_all;
use riscv::register::satp;
use spin::Mutex;
use crate::mm::translate_writable_va;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<Mutex<MemorySet>> =
        Arc::new(Mutex::new(MemorySet::new_kernel()));
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

pub fn kernel_token() -> usize {
    KERNEL_SPACE.lock().token()
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    /// Assume that no conflicts.
    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }
    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self
            .areas
            .iter_mut()
            .enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
    }
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    fn map_unfi_sche_buffer(&mut self, ppa: PhysAddr) {
        self.page_table.map(
            VirtAddr::from(UNFI_SCHE_BUFFER).into(),
            PhysPageNum::from(ppa).into(),
            PTEFlags::R | PTEFlags::W | PTEFlags::U
        );
    }
    /// Without kernel stacks.
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        debug!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        debug!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        debug!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        debug!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        debug!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        debug!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        debug!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        debug!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        debug!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        debug!("mapping plic");
        memory_set.push(
            MapArea::new(
                (0xc00_0000 as usize).into(),
                (0x1000_0000 as usize).into(),
                MapType::Mmio,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        debug!("mapping uart");
        use crate::uart;
        #[cfg(any(feature = "board_qemu", feature = "board_lrv"))]
        memory_set.push(
            MapArea::new(
                (uart::SERIAL_BASE_ADDRESS).into(),
                (uart::SERIAL_BASE_ADDRESS + uart::SERIAL_NUM * uart::SERIAL_ADDRESS_STRIDE).into(),
                MapType::Mmio,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        debug!("mapping trace");
        memory_set.push(
            MapArea::new(
                MEMORY_END.into(),
                (MEMORY_END + TRACE_SIZE).into(),
                MapType::Mmio,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        unsafe { asm!("fence.i") }
        memory_set
    }
    /// Include sections in elf and trampoline and TrapContext and user stack,
    /// also returns user_sp and entry point.
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize, usize) {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        memory_set.add_user_module(&crate::lkm::UNFI_SCHE_MEMORYSET);

        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // map user stack with U flags
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        // guard page
        user_stack_bottom += PAGE_SIZE;
        // let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        // memory_set.push(
        //     MapArea::new(
        //         user_stack_bottom.into(),
        //         user_stack_top.into(),
        //         MapType::Framed,
        //         MapPermission::R | MapPermission::W | MapPermission::U,
        //     ),
        //     None,
        // );
        // // map TrapContext
        // memory_set.push(
        //     MapArea::new(
        //         TRAP_CONTEXT.into(),
        //         TRAMPOLINE.into(),
        //         MapType::Framed,
        //         MapPermission::R | MapPermission::W,
        //     ),
        //     None,
        // );
        // map trace
        memory_set.push(
            MapArea::new(
                MEMORY_END.into(),
                (MEMORY_END + TRACE_SIZE).into(),
                MapType::Mmio,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        debug!("map unfi_sche buffer: {:#x}", UNFI_SCHE_BUFFER);
        let data_section_vir_addr = elf.find_section_by_name(".data").unwrap().address() as usize;
        let data_section_phy_addr = memory_set.page_table.translate_va(VirtAddr::from(data_section_vir_addr)).unwrap();
        memory_set.map_unfi_sche_buffer(data_section_phy_addr);
        debug!("map unfi_sche buffer done");
            unsafe { asm!("fence.i") }
        (
            memory_set,
            user_stack_bottom,
            elf.header.pt2.entry_point() as usize,
            data_section_vir_addr
        )
    }
    pub fn from_existed_user(user_space: &MemorySet) -> MemorySet {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        memory_set.add_user_module(&crate::lkm::UNFI_SCHE_MEMORYSET);

        // copy data sections/trap_context/user_stack
        for area in user_space.areas.iter() {
            let new_area = MapArea::from_another(area);
            memory_set.push(new_area, None);
            // copy data from another space
            if area.map_type != MapType::Mmio {
                for vpn in area.vpn_range {
                    let src_ppn = user_space.translate(vpn).unwrap().ppn();
                    let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                    dst_ppn
                        .get_bytes_array()
                        .copy_from_slice(src_ppn.get_bytes_array());
                }
            }
        }
        unsafe { asm!("fence.i") }
        memory_set
    }
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            sfence_vma_all();
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    fn is_mapped_area(&self, start_va: VirtAddr, end_va: VirtAddr) -> bool {
        for area in &self.areas {
            if area
                .vpn_range
                .is_overlapped(&VPNRange::new(start_va.into(), end_va.into()))
            {
                return true;
            }
        }
        false
    }

    pub fn mmap(&mut self, start: usize, len: usize, port: usize) -> Result<isize, isize> {
        if port & !7 != 0 || port & 7 == 0 || len > 1 << 30 {
            Err(-1)
        } else {
            let start_va: VirtAddr = VirtAddr::from(start);
            if start_va != start_va.floor().into() {
                return Err(-1);
            }
            let end_va: VirtAddr = VirtAddr::from(start + len).ceil().into();

            if self.is_mapped_area(start_va, end_va) {
                return Err(-1);
            }
            self.insert_framed_area(
                start_va,
                end_va,
                MapPermission::from_bits((port << 1 | 0b10000) as u8).unwrap(),
            );

            Ok((usize::from(end_va) - usize::from(start_va)) as isize)
        }
    }

    pub fn munmap(&mut self, start: usize, len: usize) -> Result<isize, isize> {
        let mut start_va: VirtAddr = VirtAddr::from(start);
        if start_va != start_va.floor().into() {
            return Err(-1);
        }
        let end_va: VirtAddr = VirtAddr::from(start + len).ceil().into();

        let mut to_unmap: Vec<usize> = Vec::new();

        for (i, area) in self.areas.iter().enumerate() {
            if area
                .vpn_range
                .is_overlapped(&VPNRange::new(start_va.into(), end_va.into()))
            {
                to_unmap.push(i);
            }
        }

        to_unmap.sort_by_key(|i| self.areas[*i].vpn_range.get_start());

        for i in &to_unmap {
            if start_va == self.areas[*i].vpn_range.get_start().into() {
                start_va = self.areas[*i].vpn_range.get_end().into();
            } else {
                return Err(-1);
            }
        }
        if start_va != end_va {
            return Err(-1);
        }

        to_unmap.sort_by(|l, r| r.cmp(l));

        for i in to_unmap {
            self.areas[i].unmap(&mut self.page_table);
            self.areas.remove(i);
        }

        Ok(len as isize)
    }

    pub fn mmio_map(&mut self, start: usize, len: usize, port: usize) -> Result<isize, isize> {
        if port & !7 != 0 || port & 7 == 0 || len > 1 << 30 {
            Err(-1)
        } else {
            let start_va: VirtAddr = VirtAddr::from(start);
            if start_va != start_va.floor().into() {
                return Err(-1);
            }
            let end_va: VirtAddr = VirtAddr::from(start + len).ceil().into();

            if self.is_mapped_area(start_va, end_va) {
                return Err(-1);
            }
            self.push(
                MapArea::new(
                    start_va,
                    end_va,
                    MapType::Mmio,
                    MapPermission::from_bits((port << 1 | 0b10000) as u8).unwrap(),
                ),
                None,
            );
            Ok((usize::from(end_va) - usize::from(start_va)) as isize)
        }
    }

    #[allow(unused)]
    pub fn mmio_unmap(&mut self, start: usize, len: usize) -> Result<isize, isize> {
        let mut start_va: VirtAddr = VirtAddr::from(start);
        if start_va != start_va.floor().into() {
            return Err(-1);
        }
        let end_va: VirtAddr = VirtAddr::from(start + len).ceil().into();

        let mut to_unmap: Vec<usize> = Vec::new();

        for (i, area) in self.areas.iter().enumerate() {
            if area
                .vpn_range
                .is_overlapped(&VPNRange::new(start_va.into(), end_va.into()))
            {
                to_unmap.push(i);
            }
        }

        to_unmap.sort_by_key(|i| self.areas[*i].vpn_range.get_start());

        for i in &to_unmap {
            if start_va == self.areas[*i].vpn_range.get_start().into() {
                start_va = self.areas[*i].vpn_range.get_end().into();
            } else {
                return Err(-1);
            }
        }
        if start_va != end_va {
            return Err(-1);
        }

        to_unmap.sort_by(|l, r| r.cmp(l));

        for i in to_unmap {
            self.areas[i].unmap(&mut self.page_table);
            self.areas.remove(i);
        }

        Ok(len as isize)
    }

    pub fn recycle_data_pages(&mut self) {
        //*self = Self::new_bare();
        self.areas.clear();
    }

    /// 得到模块的地址空间
    pub fn from_module(elf_data: &[u8]) -> Self {
        let mut memory_set = Self::new_bare();
        // map program headers of elf, with U flag
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        debug!("module entry: {:#x}", elf.header.pt2.entry_point() as usize);
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                // println!("module_va {:#x} ~ {:#x}", start_va.0, end_va.0);
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        // (
        memory_set
            // elf.header.pt2.entry_point() as usize,
        // )
    }
    pub fn add_kernel_module(&mut self, module_space: &MemorySet) {
        for area in module_space.areas.iter() {
            // crate::println!("addr {:#x?} - {:#x?}", area.vpn_range.get_start(), area.vpn_range.get_end());
            for vpn in area.vpn_range {
                // println!("ppn {:#x?}", module_space.translate(vpn).unwrap().ppn());
                self.page_table.map(
                    vpn,
                    module_space.translate(vpn).unwrap().ppn(),
                    PTEFlags::R | PTEFlags::X | PTEFlags::W,
                );
            }
        }

        self.push(
            MapArea::new(
                UNFI_SCHE_BUFFER.into(),
                (UNFI_SCHE_BUFFER + PAGE_SIZE).into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
    }
    pub fn add_user_module(&mut self, module_space: &MemorySet) {
        for area in module_space.areas.iter() {
            // println!("addr {:#x?} - {:#x?}", area.vpn_range.get_start(), area.vpn_range.get_end());
            for vpn in area.vpn_range {
                // println!("ppn {:#x?}", module_space.translate(vpn).unwrap().ppn());
                self.page_table.map(
                    vpn,
                    module_space.translate(vpn).unwrap().ppn(),
                    PTEFlags::R | PTEFlags::X | PTEFlags::W | PTEFlags::U,
                );
            }
        }
    }
}

pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    pub fn from_another(another: &MapArea) -> Self {
        Self {
            vpn_range: VPNRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
            data_frames: BTreeMap::new(),
            map_type: another.map_type,
            map_perm: another.map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Mmio => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
                trace!("map_one: vpn {:?} ppn {:?}", vpn, ppn);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        if let MapType::Framed = self.map_type {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
    Mmio,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.lock();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_text.floor())
            .unwrap()
            .writable(),
        false
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_rodata.floor())
            .unwrap()
            .writable(),
        false,
    );
    assert_eq!(
        kernel_space
            .page_table
            .translate(mid_data.floor())
            .unwrap()
            .executable(),
        false,
    );
    debug!("remap_test passed!");
}
